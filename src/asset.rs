use bitcoin_hashes::sha256d;
use bitcoin::consensus::encode::*;

use crate::constants::*;
use crate::contract::Contract;
use crate::proof::Proof;

#[derive(Clone, Debug)]
pub struct Asset {
    pub contract: Contract,
    pub proof: Proof,
}

impl Asset {
    pub fn get_asset_id(&self) -> AssetId {
        self.contract.get_asset_id()
    }

    pub fn get_root_proof(&self) -> Option<Proof> {

    }
}

impl<S: Encoder> Encodable<S> for Asset {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {

    }
}

impl<D: Decoder> Decodable<D> for Asset {
    fn consensus_decode(d: &mut D) -> Result<Asset, Error> {
        let mut asset = Asset();
        Ok(asset)
    }
}
