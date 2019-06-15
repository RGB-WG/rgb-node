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
use crate::constants::AssetId;

/// In-memory representation of offchain proofs for RGB transaction linked to an on-chain
/// bitcoin transaction. Data structure provides serialization with consensus serialization methods
/// for disk storage and network messaging between Bifröst servers and RGB-enabled wallets,
/// verification of the proof internal consistency, compliance with original asset issuing contract,
/// and tool methods for generating bitcoin output scripts for the associated on-chain transactions.
#[derive(Clone, Debug)]
pub struct Proof {
    /// Proofs of the previous RGB transaction which outputs are spent by this transaction.
    /// For the first transaction spending initial contract issuance contains an empty Vec.
    pub inputs: Vec<Proof>,

    /// Set of new transaction outputs
    pub outputs: Vec<RgbOutEntry>,

    /// Additional blueprint-specific metadata required by the contract (like scripting)
    pub metadata: Vec<u8>,

    /// Bitcoin on-chain transaction and vout to which this RGB transaction is bound to.
    pub bind_to: Vec<OutPoint>,

    /// Contract issuing assets which this RGB transaction operates
    pub contract: Option<Box<Contract>>, // Only needed for root proofs

    /// Original public key used for signing the transaction output.
    /// For pay-to-contract schemes only.
    pub original_commitment_pk: Option<PublicKey>,
}

impl<S: Encoder> Encodable<S> for Proof {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.inputs.consensus_encode(s)?;
        self.outputs.consensus_encode(s)?;
        self.metadata.consensus_encode(s)?;
        self.bind_to.consensus_encode(s)?;

        // For optionals, we use first byte to determine presence of the value (0x0 for no value,
        // 0x1 for some value) and then, if there is a value presented, we serialize it.
        match self.contract {
            Some(contract) => {
                u8(0x1).consensus_encode(s);
                contract.consensus_encode(s)?;
            },
            None => {
                u8(0x0).consensus_encode(s);
            }
        }
        match self.original_commitment_pk {
            Some(pk) => {
                u8(0x1).consensus_encode(s);
                pk.consensus_encode(s)
            },
            None => {
                u8(0x0).consensus_encode(s)
            }
        }
    }
}

impl<D: Decoder> Decodable<D> for Proof {
    fn consensus_decode(d: &mut D) -> Result<Proof, Error> {
        let inputs: Vec<Proof> = Decodable::consensus_decode(d)?;
        let outputs: Vec<RgbOutEntry> = Decodable::consensus_decode(d)?;
        let metadata: Vec<u8> = Decodable::consensus_decode(d)?;
        let bind_to: Vec<OutPoint> = Decodable::consensus_decode(d)?;

        // For optionals, we use first byte to determine presence of the value (0x0 for no value,
        // 0x1 for some value) and then, if there is a value presented, we deserialize it.
        let mut contract: Option<Box<Contract>> = None;
        if Decodable::consensus_decode(d)? {
            let c: Contract = Decodable::consensus_decode(d)?;
            contract = Some(Box(c));
        }
        let mut original_pk: Option<PublicKey> = None;
        if Decodable::consensus_decode(d)? {
            let pk = Decodable::consensus_decode(d)?;
            original_pk = Some(pk);
        }

        Ok(Proof(inputs, outputs, metadata, bind_to, contract, original_pk))
    }
}
