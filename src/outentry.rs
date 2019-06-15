use bitcoin::consensus::encode::*;
use crate::{AssetId, RgbOutPoint};

#[derive(Clone, Debug)]
pub struct RgbOutEntry {
    asset_id: AssetId,
    amount: u64,
    out_point: RgbOutPoint
}

impl<S: Encoder> Encodable<S> for RgbOutEntry {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {

    }
}

impl<D: Decoder> Decodable<D> for RgbOutEntry {
    fn consensus_decode(d: &mut D) -> Result<RgbOutEntry, Error> {
        let mut output_entry = OutputEntry();
        Ok(output_entry)
    }
}
