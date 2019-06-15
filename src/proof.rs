use bitcoin::OutPoint;
use bitcoin::consensus::encode::*;
use secp256k1::PublicKey;

use crate::{AssetId, Contract, RgbOutEntry};

#[derive(Clone, Debug)]
pub struct Proof {
    pub inputs: Vec<Proof>,
    pub outputs: Vec<RgbOutEntry>,
    pub metadata: Vec<u8>,

    pub bind_to: Vec<OutPoint>,
    pub contract: Option<Box<Contract>>, // Only needed for root proofs
    pub original_commitment_pk: Option<PublicKey>,
}

impl<S: Encoder> Encodable<S> for Proof {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {

    }
}

impl<D: Decoder> Decodable<D> for Proof {
    fn consensus_decode(d: &mut D) -> Result<Proof, Error> {
        let mut proof = Proof();
        Ok(proof)
    }
}
