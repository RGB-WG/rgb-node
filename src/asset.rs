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

//! RGB asset abstractions

use std::io::Cursor;

use bitcoin::consensus::encode::*;

use crate::constants::*;
use crate::contract::{Contract, ContractBody, ReissueContractBody};
use crate::proof::Proof;

/// RGB asset data structure for in-memory representation of bundled asset issuence contracts and
/// chain of proofs for each of the known assets
#[derive(Clone, Debug)]
pub struct Asset<B: ContractBody> {
    /// Original asset issue contract
    pub contract: Contract<B>,

    /// Set of asset reissue contracts (if any)
    pub reissues: Vec<Contract<ReissueContractBody>>,

    /// Set of all known unspent RGB proofs for a given asset (i.e. heads of the proof chains)
    pub proof_chains: Vec<Proof<B>>,
}

impl<B: ContractBody> Asset<B> where B: Encodable<Cursor<Vec<u8>>> {
    /// Provides unique asset_id, which is computed as a SHA256d-hash from the consensus-serialized
    /// contract data
    pub fn get_asset_id(&self) -> AssetId {
        self.contract.get_asset_id()
    }
}

impl<S: Encoder, T: Encodable<S> + ContractBody> Encodable<S> for Asset<T> {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.contract.consensus_encode(s)?;
        self.reissues.consensus_encode(s)?;
        self.proof_chains.consensus_encode(s)
    }
}

impl<D: Decoder, T: Decodable<D> + ContractBody> Decodable<D> for Asset<T> {
    fn consensus_decode(d: &mut D) -> Result<Asset<T>, Error> {
        let contract: Contract<T> = Decodable::consensus_decode(d)?;
        let reissues: Vec<Contract<ReissueContractBody>> = Decodable::consensus_decode(d)?;
        let proof_chains: Vec<Proof<T>> = Decodable::consensus_decode(d)?;
        Ok(Asset {contract, reissues, proof_chains})
    }
}
