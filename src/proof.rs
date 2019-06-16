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
use std::io::{Write, Cursor};

use bitcoin_hashes::{sha256d, hash160, Hash};
use bitcoin::OutPoint;
use bitcoin::blockdata::{opcodes::all::*, script::Builder, script::Script};
use bitcoin::consensus::encode::*;
use secp256k1::PublicKey;

use crate::{IdentityHash, Contract, CommitmentScheme, RgbOutEntry, RgbError};
use crate::contract::ContractBody;

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

    /// Bitcoin on-chain transaction and vout to which this RGB transaction is bound to.
    pub bind_to: Vec<OutPoint>,

    /// Contract issuing assets which this RGB transaction operates
    pub contract: Option<Box<Contract<T>>>, // Only needed for root proofs

    /// Original public key used for signing the transaction output.
    /// For pay-to-contract schemes only.
    pub original_commitment_pk: Option<PublicKey>,
}

impl<T: ContractBody + Encodable<Cursor<Vec<u8>>>> Proof<T> {
    /// Function providing unique hash ID of the RGB transaction proof. It is based on
    /// serialization of all transaction outputs
    pub fn get_identity_hash(&self) -> IdentityHash {
        // We do really need to hash not only outputs (as was done in the reference implementation),
        // since one can master an attack vector targeting the same UTXOs with the same amounts
        // with some fake asset, and it will have the same hash for this setting...
        let mut data = serialize(&self.outputs);
        data.extend(&serialize(&self.inputs));
        data.extend(&serialize(&self.bind_to));
        data.extend(&serialize(&self.metadata));
        match self.contract {
            Some(ref boxed) => {
                data.extend(&serialize(&0x1));
                data.extend(&serialize(&*boxed));
            },
            None => {
                data.extend(&serialize(&0x0));
            }
        }
        sha256d::Hash::from_slice(&data[..]).unwrap()
    }

    /// Returns root RGB contract for the proof by iterating through all of the first ascendants
    /// of the proof (using `inputs`)
    pub fn get_contract(&self) -> Result<&Contract<T>, RgbError<T>> {
        match self.contract {
            Some(ref boxed) => Ok(&**boxed),
            None => Err(RgbError::ProofWithoutContract(self))
        }
    }

    /// Generates Bitcoin script corresponding to the proof (taking into account the commitment
    /// scheme of the source RGB contract)
    pub fn get_script(&self) -> Result<Script, RgbError<T>> {
        let builder = match self.get_contract()?.header.commitment_scheme {
            CommitmentScheme::OpReturn => Builder::new()
                // Simple OP_RETURN with data corresponding to the hash of the current proof
                .push_opcode(OP_RETURN)
                .push_slice(&self.get_identity_hash().into_inner()),

            CommitmentScheme::PayToContract => Builder::new()
                // Pay to contract: standard P2PKH utilizing tweaked public key
                .push_opcode(OP_DUP)
                .push_opcode(OP_HASH160)
                .push_slice(
                    &sha256d::Hash::from_engine({
                            let mut engine = sha256d::Hash::engine();
                            engine.write_all(
                                &self.original_commitment_pk
                                    .ok_or_else(|| RgbError::NoOriginalPubKey(self))?
                                    .serialize()
                            );
                            engine
                        }
                    )[..]
                )
                .push_opcode(OP_EQUALVERIFY)
                .push_opcode(OP_CHECKSIG),

            _ => return Err(RgbError::ProofWithoutContract(self)),
        };
        Ok(builder.into_script())
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
                let data = pk.serialize();
                data.consensus_encode(s)
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
        let bind_to: Vec<OutPoint> = Decodable::consensus_decode(d)?;

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
