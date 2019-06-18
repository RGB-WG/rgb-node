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

use crate::{Proof, CommitmentScheme};
use crate::contract::ContractBody;
use crate::constants::IdentityHash;

///! Error types for RGB protocol
pub enum RgbError<'a, T: ContractBody> {
    BitcoinHashError(BitcoinHashError),
    IoError(io::Error),

    UnsupportedCommitmentScheme(CommitmentScheme),
    NoOriginalPubKey(IdentityHash),
    ProofWithoutContract(&'a Proof<T>),
}

impl<'a, T: ContractBody + Encodable<Cursor<Vec<u8>>>> Display for RgbError<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            RgbError::ProofWithoutContract(id) =>
                write!(f, "Proof {} does not have a root contract", **id),
            RgbError::UnsupportedCommitmentScheme(ref scheme) =>
                write!(f, "Unknown commitment scheme with id {}", { let s: u8 = scheme.clone().into(); s }),
            RgbError::BitcoinHashError(err) => Display::fmt(err, f),
            RgbError::IoError(err) => Display::fmt(err, f),
            RgbError::NoOriginalPubKey(ref hash) =>
                write!(f, "No original pubkey found pay-to-contract proof {}", *hash),
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
