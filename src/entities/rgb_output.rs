use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::util::uint::Uint256;
use entities::rgb_output::RgbOutPoint::{KnownUTXO, NewUTXO};
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::util::hash::Sha256dHash;
use bitcoin::util::hash::Sha256dEncoder;

#[derive(Copy, Clone, Debug)]
pub struct RgbOutput {
    pub amount: u32,
    pub token_id: Sha256dHash,
    pub to_output: RgbOutPoint,
}

#[derive(Copy, Clone, Debug)]
pub enum RgbOutPoint {
    KnownUTXO(Sha256dHash),
    NewUTXO(u16),
}

impl RgbOutput {
    pub fn new(amount: u32, token_id: Sha256dHash, to_output: RgbOutPoint) -> RgbOutput {
        RgbOutput {
            amount,
            token_id,
            to_output,
        }
    }
}

impl RgbOutPoint {
    pub fn new_utxo(utxo: TxOutRef) -> RgbOutPoint {
        KnownUTXO(RgbOutPoint::hash_txoutref(utxo))
    }

    pub fn hash_txoutref(utxo: TxOutRef) -> Sha256dHash {
        let mut enc = Sha256dEncoder::new();

        utxo.txid.consensus_encode(&mut enc).unwrap();
        (utxo.index as u32).consensus_encode(&mut enc).unwrap(); // TODO: Is the casting here really necessary?

        enc.into_hash()
    }

    pub fn new_index(index: u16) -> RgbOutPoint {
        NewUTXO(index)
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for RgbOutput {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        self.amount.consensus_encode(s)?;
        self.token_id.consensus_encode(s)?;
        self.to_output.consensus_encode(s)?;

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for RgbOutput {
    fn consensus_decode(d: &mut D) -> Result<RgbOutput, D::Error> {
        Ok(RgbOutput {
            amount: ConsensusDecodable::consensus_decode(d)?,
            token_id: ConsensusDecodable::consensus_decode(d)?,
            to_output: ConsensusDecodable::consensus_decode(d)?,
        })
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for RgbOutPoint {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            RgbOutPoint::KnownUTXO(ref utxo_hash) => {
                (1u8).consensus_encode(s)?; // known utxo, 0x01
                utxo_hash.consensus_encode(s)?;
            }
            RgbOutPoint::NewUTXO(index) => {
                (2u8).consensus_encode(s)?; // new utxo, 0x02
                index.consensus_encode(s)?; // index
            }
        }

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for RgbOutPoint {
    fn consensus_decode(d: &mut D) -> Result<RgbOutPoint, D::Error> {
        let rgb_outpoint_kind: u8 = ConsensusDecodable::consensus_decode(d)?;

        match rgb_outpoint_kind {
            0x01 => {
                Ok(
                    RgbOutPoint::KnownUTXO(ConsensusDecodable::consensus_decode(d)?)
                )
            }
            0x02 => {
                Ok(
                    RgbOutPoint::NewUTXO(ConsensusDecodable::consensus_decode(d)?)
                )
            }
            x => Err(d.error(format!("RgbOutPoint kind {:02x} not understood", x)))
        }
    }
}