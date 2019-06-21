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

//! RGB contracts
//!
//! Implementation of data structures used in RGB contracts

use std::fmt;
use std::rc::Weak;
use std::io::Cursor;

use bitcoin_hashes::{sha256d, Hash};
use bitcoin::consensus::encode::*;
use secp256k1::PublicKey;

use crate::*;

/// Commitment scheme variants used by RGB contract header field `commitment_scheme`.
/// With the current specification only two possible schemes are supported: OP_RETURN and
/// pay-to-contract. See more at <https://github.com/rgb-org/spec/blob/master/01-rgb.md#commitment-scheme>
///
/// NB: Commitment scheme specifies the way of commiting proofs for RGB transactions, not
/// the way by which original RGB contract is commited
#[repr(u8)]
#[derive(Clone, Debug)]
pub enum CommitmentScheme {
    /// Used by reissuance blueprint contract, which inherits `commitment_scheme` from
    /// the original issuance contract.
    NotApplicable,

    /// OP_RETURN scheme, committing RGB proofs to a special bitcoin transaction output
    /// containing `OP_RETURN` opcode followed by the hash of RGB contract or proof
    OpReturn,

    /// Pay to contract scheme, committing RGB proofs to a bitcoin UTXO via public key tweak.
    PayToContract,
}

impl From<u8> for CommitmentScheme {
    fn from(no: u8) -> Self {
        match no {
            0x1 => CommitmentScheme::OpReturn,
            0x2 => CommitmentScheme::PayToContract,
            _ => CommitmentScheme::NotApplicable,
        }
    }
}

impl From<CommitmentScheme> for u8 {
    fn from(scheme: CommitmentScheme) -> Self {
        match scheme {
            CommitmentScheme::OpReturn => 0x1,
            CommitmentScheme::PayToContract => 0x2,
            CommitmentScheme::NotApplicable => 0x0,
        }
    }
}


/// Types of blueprints for the RGB contracts. Each blueprint type defines custom fields used
/// in the contract body – and sometimes special requirements for the contract header fields.
/// Read more on <https://github.com/rgb-org/spec/blob/master/01-rgb.md#blueprints-and-versioning>
///
/// Subjected to the future extension, at this moment this is very preliminary work.
#[repr(u16)]
#[derive(Clone, Debug)]
pub enum BlueprintType {
    /// Simple issuance contract
    Issue,

    /// Crowdsale contract
    Crowdsale,

    /// Reissuing contract
    Reissue,

    /// Reserved for all other blueprints which are unknown for the current version
    Unknown,
}

impl From<u16> for BlueprintType {
    fn from(no: u16) -> Self {
        match no {
            0x0001 => BlueprintType::Issue,
            0x0002 => BlueprintType::Crowdsale,
            0x0003 => BlueprintType::Reissue,
            _ => BlueprintType::Unknown,
        }
    }
}


impl From<BlueprintType> for u16 {
    fn from(blueprint: BlueprintType) -> Self {
        match blueprint {
            BlueprintType::Issue => 0x0001,
            BlueprintType::Crowdsale => 0x0002,
            BlueprintType::Reissue => 0x0003,
            BlueprintType::Unknown => 0xFFFF,
        }
    }
}

/// RGB Contract in-memory representation.
///
/// Data structure provides serialization with consensus serialization methods
/// for disk storage and network messaging between Bifröst servers and RGB-enabled wallets,
/// verification of the contract internal consistency and blueprint specification
/// and tool methods for generating bitcoin output scripts for the associated on-chain transaction.
#[derive(Clone, Debug)]
pub struct Contract<B: ContractBody> {
    /// Contract header, containing fixed set of fields, shared by all contract blueprints
    pub header: ContractHeader,

    /// Contract body, with blueprint-specific set of fields
    pub body: Box<B>,

    /// Original public key used for signing the contract. Used for pay-to-contract schemes only.
    /// Serialized, but not a part of the commitment hash.
    pub original_commitment_pk: Option<PublicKey>,

    /// Contract must weakly reference it's root proof. Since it's unknown during deserealization,
    /// it is defined as optional; however it must contain value when the contract is used,
    /// otherwise a `ContractWithoutRootProof` error will be produced.
    pub root_proof: Option<Weak<Proof<B>>>,
}

impl<B: ContractBody> Contract<B> where B: Encodable<Cursor<Vec<u8>>> {
    /// Validates given proof to have a correct structure for the used RGB contract blueprint
    /// (i.e. it has or has no metadata field, has original public key for pay-to-contract
    /// commitment schemes etc.)
    pub fn validate_proof<'a>(&'a self, proof: &'a Proof<B>) -> Result<(), RgbError<'a, B>> {
        // Validate that the proof is matching header fields
        self.header.validate_proof(proof)?;
        // Validate proof regarding custom contract blueprint (like correct metadata scripts etc)
        self.body.validate_proof(proof)
    }
}

impl<B: ContractBody> OnChain<B> for Contract<B> where B: Encodable<Cursor<Vec<u8>>> {
    /// Provides unique get_identity_hash, which is computed as a SHA256d-hash from the
    /// consensus-serialized contract data, prefixed with 'rgb' due to
    /// <https://github.com/rgb-org/spec/issues/61>
    fn get_identity_hash(&self) -> IdentityHash {
        let mut hash: Vec<u8> = "rgb".into();
        // We can't use standard serialization of the contract from bitcoin::Encodable trait
        // since here we do not need to commit to `original_commitment_pk` field, which is
        // serialized by the trait implementation for disk storage and network transfers.
        // So here we are doing custom serialization of contract header and body.
        hash.extend(serialize(&self.header));
        hash.extend(serialize(&*self.body));
        sha256d::Hash::from_slice(hash.as_slice()).unwrap()
    }

    /// Returns RGB contract, i.e. itself
    fn get_contract(&self) -> Result<&Contract<B>, RgbError<B>> {
        Ok(&self)
    }

    /// Returns untweaked public key if the pay-to-contract commitment scheme is used.
    fn get_original_pk(&self) -> Option<PublicKey> {
        self.original_commitment_pk
    }
}

impl<B: ContractBody + Verify<B>> Verify<B> for Contract<B> {
    /// Function performing verification of the integrity for the RGB contract for both on-chain
    /// and off-chain parts; including internal consistency, integrity,  proper formation of
    /// commitment transactions etc.
    ///
    /// # Arguments:
    /// * `tx_provider` - a specially-formed callback function provided by the callee (wallet app
    /// or bifrost server) that returns transaction for a given case (specified by `TxQuery`-typed
    /// argument given to the callback). Used during the verification process to check on-chain
    /// part of the contract. Since rgblib has no direct access to a bitcoin node
    /// (it's rather a task for particular wallet or Bifrost implementation) it relies on this
    /// callback during the verification process.
    fn verify(&self, tx_provider: TxProvider<B>) -> Result<(), RgbError<B>> {
        self.header.verify(tx_provider)?;
        self.body.verify(tx_provider)
    }
}

impl<T: ContractBody + Encodable<Cursor<Vec<u8>>>> fmt::Display for Contract<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.get_identity_hash())
    }
}

impl<S: Encoder, T: Encodable<S> + ContractBody> Encodable<S> for Contract<T> {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.header.consensus_encode(s)?;
        (*self.body).consensus_encode(s)?;

        // We do not need to serialize a flag whether `original_commitment_pk` is present since
        // its presence is defined by the `commitment_scheme` field in the contract header,
        // which is already serialized
        match self.original_commitment_pk {
            Some(pk) => pk.serialize().consensus_encode(s),
            None => Ok(()),
        }
    }
}

impl<D: Decoder, T: Decodable<D> + ContractBody> Decodable<D> for Contract<T> {
    fn consensus_decode(d: &mut D) -> Result<Contract<T>, Error> {
        let header: ContractHeader = Decodable::consensus_decode(d)?;
        let body: Box<T> = Box::new(Decodable::consensus_decode(d)?);
        let mut original_commitment_pk: Option<PublicKey> = None;
        match header.commitment_scheme {
            CommitmentScheme::PayToContract => {
                let data: Vec<u8> = Decodable::consensus_decode(d)?;
                match PublicKey::from_slice(&data[..]) {
                    Ok(pk) => original_commitment_pk = Some(pk),
                    Err(_) => return Err(
                        bitcoin::consensus::encode::Error::ParseFailed("Can't decode public key"))
                };
            },
            _ => ()
        };

        Ok(Contract{ header, body, original_commitment_pk, root_proof: None })
    }
}
