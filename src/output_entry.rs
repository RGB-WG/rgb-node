use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::util::hash::Sha256dHash;

/// RGB output
#[derive(Clone, Debug)]
pub struct OutputEntry(
    /// Asset id
    Sha256dHash,
    /// Asset amount
    u32,
    /// Vout (optional): the index of this RGB output as bitcoin transaction output (?)
    Option<u32>);

impl OutputEntry {
    pub fn new(asset_id: Sha256dHash, amount: u32, vout: Option<u32>) -> OutputEntry {
        OutputEntry(asset_id, amount, vout)
    }

    pub fn get_asset_id(&self) -> Sha256dHash {
        self.0.clone()
    }

    pub fn get_amount(&self) -> u32 {
        self.1
    }

    pub fn get_vout(&self) -> Option<u32> {
        self.2
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for OutputEntry {
    fn consensus_encode(&self, s: &mut S) -> Result<(), serialize::Error> {
        self.0.consensus_encode(s)?;
        self.1.consensus_encode(s)?;
        self.2.consensus_encode(s)
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for OutputEntry {
    fn consensus_decode(d: &mut D) -> Result<OutputEntry, serialize::Error> {
        Ok(OutputEntry::new(ConsensusDecodable::consensus_decode(d)?, ConsensusDecodable::consensus_decode(d)?, ConsensusDecodable::consensus_decode(d)?))
    }
}