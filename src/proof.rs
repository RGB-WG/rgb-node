// RGB Rust Library
// Written in 2019 by
//     Dr. Maxim Orlovsky <dr.orlovsky@gmail.com>
// basing on ideas from the original RGB rust library by
//     Alekos Filini <alekos.filini@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

//! RGB transaction proofs for off-chain storage
//!
//! Implementation of data structures used in RGB transaction proofs


use std::fmt;
use std::io::Cursor;

use bitcoin_hashes::{sha256d, Hash};
use bitcoin::{Transaction, OutPoint};
use bitcoin::consensus::encode::*;
use secp256k1::PublicKey;

use crate::{IdentityHash, OnChain, Contract,
            TxQuery, TxProvider, RgbTransaction, RgbOutEntry, RgbOutPoint, RgbError};
use crate::contract::ContractBody;
use std::collections::HashMap;
use crate::constants::AssetId;

/// In-memory representation of offchain proofs for RGB transaction linked to an on-chain
/// bitcoin transaction. Data structure provides serialization with consensus serialization methods
/// for disk storage and network messaging between Bifröst servers and RGB-enabled wallets,
/// verification of the proof internal consistency, compliance with original asset issuing contract,
/// and tool methods for generating bitcoin output scripts for the associated on-chain transactions.
#[derive(Clone, Debug)]
pub struct Proof<T: ContractBody> {
    /// Proofs of the previous RGB transaction which outputs are spent by this transaction.
    /// For the first transaction spending initial contract issuance contains an empty Vec.
    pub inputs: Vec<Proof<T>>,

    /// Set of new transaction outputs
    pub outputs: Vec<RgbOutEntry>,

    /// Additional blueprint-specific metadata required by the contract (like scripting)
    pub metadata: Vec<u8>,

    /// Bitcoin on-chain transaction and its vouts to which this RGB transaction is bound to.
    pub bind_to: RgbTransaction,

    /// Contract issuing assets which this RGB transaction operates
    pub contract: Option<Box<Contract<T>>>, // Only needed for root proofs

    /// Original public key used for signing the transaction output.
    /// For pay-to-contract schemes only.
    pub original_commitment_pk: Option<PublicKey>,
}

impl<B: ContractBody> Proof<B> {
    /// Checks if this proof is a root proof. The root proof has no upstream proofs (i.e. empty
    /// `inputs` list) and must have an associated contract. The function checks both conditions
    /// and returns `true` only if they are both satisfied.
    pub fn is_root(&self) -> bool {
        self.inputs.len() == 0 && self.contract.is_some()
    }

    /// Computes and returns sum of all input amounts as per asset (packed in a HashMap)
    pub fn get_input_amounts(&self) -> HashMap<AssetId, u64> {
        // For the root proof we need to return total supply
        if self.is_root() {
            let assets = HashMap::new();
            assets[self.get_identity_hash()] = self.contract.unwrap().header.total_supply;
        }
        // Otherwise we compute sums for each asset by iterating over inputs
        let init: HashMap<AssetId, u64> = HashMap::new();
        self.inputs.iter().fold(init, |mut acc, input| {
            input.get_input_amounts().iter().for_each(|input_amounts| {
                *acc.entry(*input_amounts.0).or_insert(0) += input_amounts.1;
            });
            acc
        })
    }

    /// Computes and returns sum of all output amounts as per asset (packed in a HashMap)
    pub fn get_output_amounts(&self) -> HashMap<AssetId, u64> {
        let init: HashMap<AssetId, u64> = HashMap::new();
        self.outputs.iter().fold(init, |mut acc, output| {
            *acc.entry(output.asset_id).or_insert(0) += output.amount;
            acc
        })
    }
}

impl<B: ContractBody> OnChain<B> for Proof<B> where B: Encodable<Cursor<Vec<u8>>> {
    /// Function providing unique hash ID of the RGB transaction proof. It is based on
    /// serialization of all transaction outputs
    fn get_identity_hash(&self) -> IdentityHash {
        // We do really need to hash not only outputs (as was done in the reference implementation),
        // since one can master an attack vector targeting the same UTXOs with the same amounts
        // with some fake asset, and it will have the same hash for this setting...
        let mut data = serialize(&self.outputs);
        data.extend(serialize(&self.inputs));
        data.extend(serialize(&self.bind_to));
        data.extend(serialize(&self.metadata));
        match self.contract {
            Some(ref boxed) => {
                data.extend(serialize(&0x1));
                data.extend(serialize(&*boxed));
            },
            None => {
                data.extend(serialize(&0x0));
            }
        }
        sha256d::Hash::from_slice(&data[..]).unwrap()
    }

    /// Returns root RGB contract for the proof by iterating through all of the first ascendants
    /// of the proof (using `inputs`)
    fn get_contract(&self) -> Result<&Contract<B>, RgbError<B>> {
        match self.contract {
            Some(ref boxed) => Ok(&**boxed),
            None => Err(RgbError::ProofWithoutContract(self))
        }
    }

    /// Returns untweaked public key if the pay-to-contract commitment scheme is used in the
    /// RGB contract; `None` otherwise
    fn get_original_pk(&self) -> Option<PublicKey> {
        self.original_commitment_pk
    }

    /// Function performing verification of the integrity for the RGB proof
    /// for both on-chain and off-chain parts; including internal consistency, integrity,
    /// proper formation of commitment transactions etc. The function iterates over all proof chain
    /// up to the root proof, and also verifies RGB contract associated with the root proof by
    /// calling the same `verify` method for the contract struct.
    ///
    /// # Arguments:
    /// * `tx_provider` - a specially-formed callback function provided by the callee (wallet app
    /// or bifrost server) that returns transaction for a given case (specified by `TxQuery`-typed
    /// argument given to the callback). Used during the verification process to check on-chain
    /// part of the proof. Since rgblib has no direct access to a bitcoin node
    /// (it's rather a task for particular wallet or Bifrost implementation) it relies on this
    /// callback during the verification process.
    fn verify(&self, tx_provider: TxProvider<B>) -> Result<(), RgbError<B>> {
        // 1. Checking proof integrity
        match (&self.contract, self.inputs.is_empty()) {
            // 1.1. Proofs with contract assigned must be root proofs
            (Some(_), false) =>
                return Err(RgbError::ProofWithoutContract(self)),

            // 1.2. All non-root proofs MUST have upstream proofs
            (None, true) =>
                return Err(RgbError::ProofWihoutInputs(self)),

            // 2. Contract must pass verification check
            (Some(ref contract), true) =>
                contract.verify(tx_provider)?,

            // Noting to check in all other cases, they are fine
            _ => (),
        }

        // 3. Validate that proof has correct structure for the given RGB contract blueprint
        // (i.e. has or has no metadata field, has original public key for pay-to-contract
        // commitment schemes etc.)
        self.get_contract()?.validate_proof(&self)?;

        // 4. Re-iterate verification for each of the upstream proofs
        self.inputs.iter().try_for_each(|proof| {
            proof.verify(tx_provider)
        })?;

        // 5. Verify associated commitments in bitcoin transactions
        let commitment_tx = tx_provider(TxQuery::TxId(self.bind_to.txid))?;
        self.bind_to.vouts.iter().try_for_each(|vout| {
            let vout_no = *vout as usize;
            // 5.1. Check that commitment transaction has all the necessary outputs referenced
            // by the proof
            if vout_no >= commitment_tx.output.len() {
                return Err(RgbError::MissingVout(self, *vout));
            }
            // 5.1. Check that each output referenced by the proof is colored with proper script
            if commitment_tx.output[vout_no].script_pubkey != self.get_script()? {
                return Err(RgbError::WrongScript(self, *vout));
            }
            Ok(())
        })?;

        // 6. Matching input and output balances
        let in_balances = self.get_input_amounts();
        let out_balances = self.get_output_amounts();
        // 6.1. There should be the same number of assets in both inputs and outputs
        if in_balances.len() != out_balances.len() {
            return Err(RgbError::AssetsNotEqual(self));
        }
        // 6.2. Comparing input and output amounts per each asset
        in_balances.iter().try_for_each(|inp| {
            let asset_id = inp.0;
            let in_amount = inp.1;
            match out_balances.get(asset_id) {
                Some(out_amount) => if in_amount != out_amount {
                    Err(RgbError::AmountsNotEqual(self, *asset_id))
                } else {
                    Ok(())
                },
                None => Err(RgbError::AssetsNotEqual(self)),
            }
        })?;

        // 7. Check commitment transactions for each of the proof inputs
        // 7.1. For non-root proofs there MUST be inputs for the transaction
        if self.inputs.len() == 0 && !self.is_root() {
            return Err(RgbError::NoInputs(self));
        }
        // 7.2. Check commitment transactions for each of the proof inputs
        self.inputs.iter().try_for_each(|input_proof| {
            // Getting commitment transaction for the specific proof input
            let input_tx = tx_provider(TxQuery::TxId(input_proof.bind_to.txid))?;

            // Filtering bitcoin transaction inputs for the proof commitment transaction
            // corresponding to the proof input
            let txins = commitment_tx.input.iter().filter(
                |txin| txin.previous_output.txid != input_tx.txid()
            );

            input_proof.outputs.iter().try_for_each(|output| {
                match output.out_point {
                    RgbOutPoint::UTXO(txid) => {
                        // TODO: Implement proof inputs verification for UTXO-bound RGB asset transfers
                        Ok(())
                    },
                    RgbOutPoint::Vout(vout) => {
                        // Getting those transactions which vouts matches vout specified in the
                        // input proof
                        let txins = txins.clone();
                        let correct_txins = txins.filter(
                            |txin| txin.previous_output.vout == vout
                        );
                        // There should the be only matching transaction
                        match correct_txins.count() {
                            1 => Ok(()),
                            _ => Err(RgbError::MissingVout(input_proof, vout)),
                        }
                    },
                }
            })
        })
    }
}

impl<T: ContractBody + Encodable<Cursor<Vec<u8>>>> fmt::Display for Proof<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.get_identity_hash())
    }
}

impl<S: Encoder, T: Encodable<S> + ContractBody> Encodable<S> for Proof<T> {
    fn consensus_encode(&self, s: &mut S) -> Result<(), bitcoin::consensus::encode::Error> {
        self.inputs.consensus_encode(s)?;
        self.outputs.consensus_encode(s)?;
        self.metadata.consensus_encode(s)?;
        self.bind_to.consensus_encode(s)?;

        // For optionals, we use first byte to determine presence of the value (0x0 for no value,
        // 0x1 for some value) and then, if there is a value presented, we serialize it.
        match self.contract {
            Some(ref contract) => {
                true.consensus_encode(s)?;
                contract.consensus_encode(s)?;
            },
            None => {
                false.consensus_encode(s)?;
            }
        }
        match self.original_commitment_pk {
            Some(pk) => {
                true.consensus_encode(s)?;
                pk.serialize().consensus_encode(s)
            },
            None => {
                false.consensus_encode(s)
            }
        }
    }
}

impl<D: Decoder, T: Decodable<D> + ContractBody> Decodable<D> for Proof<T> {
    fn consensus_decode(d: &mut D) -> Result<Proof<T>, bitcoin::consensus::encode::Error> {
        let inputs: Vec<Proof<T>> = Decodable::consensus_decode(d)?;
        let outputs: Vec<RgbOutEntry> = Decodable::consensus_decode(d)?;
        let metadata: Vec<u8> = Decodable::consensus_decode(d)?;
        let bind_to: RgbTransaction = Decodable::consensus_decode(d)?;

        // For optionals, we use first byte to determine presence of the value (0x0 for no value,
        // 0x1 for some value) and then, if there is a value presented, we deserialize it.
        let mut contract: Option<Box<Contract<T>>> = None;
        if Decodable::consensus_decode(d)? {
            let c: Contract<T> = Decodable::consensus_decode(d)?;
            contract = Some(Box::new(c));
        }
        let mut original_commitment_pk: Option<PublicKey> = None;
        if Decodable::consensus_decode(d)? {
            let data: Vec<u8> = Decodable::consensus_decode(d)?;
            match PublicKey::from_slice(&data[..]) {
                Ok(pk) => original_commitment_pk = Some(pk),
                Err(_) => return Err(
                        bitcoin::consensus::encode::Error::ParseFailed("Can't decode public key"))
            };
        }

        Ok(Proof { inputs, outputs, metadata, bind_to, contract, original_commitment_pk })
    }
}
