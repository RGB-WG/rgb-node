// RGB Rust Library
// Written in 2019 by
//     Dr. Maxim Orlovsky <dr.orlovsky@gmail.com>
// basing on the original RGB rust library by
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


use bitcoin::OutPoint;
use bitcoin::consensus::encode::*;
use secp256k1::PublicKey;

use crate::{Contract, RgbOutEntry};
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

impl<T: ContractBody> Proof<T> {
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
