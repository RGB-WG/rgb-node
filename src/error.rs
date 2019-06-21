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

use std::io::{self, Cursor};
use std::fmt::{Display, Formatter, Error};
use std::convert::From;

use bitcoin_hashes::error::Error as BitcoinHashError;
use bitcoin::consensus::encode::*;

use crate::{Proof, Contract, CommitmentScheme};
use crate::contract::ContractBody;
use crate::constants::{IdentityHash, AssetId};

///! Error types for RGB protocol
pub enum RgbError<'a, B: ContractBody> {
    BitcoinHashError(BitcoinHashError),
    IoError(io::Error),

    ProofWithoutContract(&'a Proof<B>),
    ContractWithoutRootProof(&'a Contract<B>),
    ProofWihoutInputs(&'a Proof<B>),
    MissingVout(&'a Proof<B>, u32),
    WrongScript(&'a Proof<B>, u32),
    AssetsNotEqual(&'a Proof<B>),
    AmountsNotEqual(&'a Proof<B>, AssetId),

    UnsupportedCommitmentScheme(CommitmentScheme),
    NoOriginalPubKey(IdentityHash),
    ProofStructureNotMatchingContract(&'a Proof<B>),
}

impl<'a, T: ContractBody + Encodable<Cursor<Vec<u8>>>> Display for RgbError<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            RgbError::BitcoinHashError(err) => Display::fmt(err, f),
            RgbError::IoError(err) => Display::fmt(err, f),

            RgbError::ProofWithoutContract(id) =>
                write!(f, "Root proof {} does not reference a contract", **id),
            RgbError::ContractWithoutRootProof(id) =>
                write!(f, "Contract {} does not reference a root proof", **id),
            RgbError::ProofWihoutInputs(id) =>
                write!(f, "Non-root proof {} does not have any upstream proofs", **id),
            RgbError::MissingVout(id, vout) =>
                write!(f, "Proof {} references unexisting output {} in its bouding tx", **id, vout),
            RgbError::WrongScript(id, vout) =>
                write!(f, "Output {} for the proof {} is not colored with a proper script",
                       **id, vout),
            RgbError::AssetsNotEqual(id) =>
                write!(f, "Input and output assets for the proof {} do not match", **id),
            RgbError::AmountsNotEqual(proof, asset_id) =>
                write!(f, "Input and output asset {} amounts for the proof {} are not equal",
                       *asset_id, **proof),

            RgbError::UnsupportedCommitmentScheme(ref scheme) =>
                write!(f, "Unknown commitment scheme with id {}",
                       { let s: u8 = scheme.clone().into(); s }),
            RgbError::NoOriginalPubKey(ref hash) =>
                write!(f, "No original public key is found pay-to-contract proof {}", *hash),
            RgbError::ProofStructureNotMatchingContract(id) =>
                write!(f, "Proof structure for {} does not match RGB contract structure", **id),
        }
    }
}

impl<'a, B: ContractBody> From<BitcoinHashError> for RgbError<'a, B> {
    fn from(err: BitcoinHashError) -> Self {
        RgbError::BitcoinHashError(err)
    }
}

impl<'a, B: ContractBody> From<io::Error> for RgbError<'a, B> {
    fn from(err: io::Error) -> Self {
        RgbError::IoError(err)
    }
}
