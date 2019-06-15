use bitcoin::consensus::encode::*;

use crate::RgbOutHash;

/// Outpoint for an RGB transaction, defined by the https://github.com/rgb-org/spec/blob/master/01-rgb.md#rgboutpoint
#[derive(Clone, Debug)]
pub enum RgbOutPoint {
    /// UTXO-based RGB transaction, pointing to the hash of some pre-existing UTXO with some `vout` in it
    UTXO(RgbOutHash),

    /// Vout-based RGB transaction, pointing to specific vout of the current bitcoin transaction
    /// containing RGB proof itself
    Vout(u32),
}

impl<S: Encoder> Encodable<S> for RgbOutPoint {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        // Encoding RgbOutPoint according to the rules specified in
        // https://github.com/rgb-org/spec/blob/master/01-rgb.md#rgboutpoint:
        // First byte — code for the type of RgbOutPoint
        match self {
            RgbOutPoint::UTXO(hash) => {
                // 0x1 stands for UTXO-based RGB transaction
                u8(0x1).consensus_encode(s)?;
                // next we put the hash of the concatenated TX hash and 32-bit vout:
                // SHA256D(TX_HASH || OUTPUT_INDEX_AS_U32)
                hash.consensus_encode(s)
            },
            RgbOutPoint::Vout(vout) => {
                // 0x2 stands for address-based RGB transaction
                u8(0x2).consensus_encode(s)?;
                // next we need to put vout to which asset will be bound
                vout.consensus_encode(s)
            },
        }
    }
}

impl<D: Decoder> Decodable<D> for RgbOutPoint {
    fn consensus_decode(d: &mut D) -> Result<RgbOutPoint, Error> {
        // Encoding RgbOutPoint according to the rules specified in
        // https://github.com/rgb-org/spec/blob/master/01-rgb.md#rgboutpoint:
        // First byte — code for the type of RgbOutPoint
        match Decodable::consensus_decode(d)? {
            // 0x1 stands for UTXO-based RGB transaction
            0x1 => {
                Ok(RgbOutPoint::UTXO(Decodable::consensus_decode(d)?))
            },
            // 0x2 stands for address-based RGB transaction
            0x2 => {
                Ok(RgbOutPoint::Vout(Decodable::consensus_decode(d)?))
            },
            // Report error in all other cases. Here we re-use one of standard bitcoin
            // serializer error types, which suits our needs well
            _ => Err(Error::ParseFailed("Wrong RGB output point type"))
        }
    }
}

#[cfg(test)]
mod test {
    use secp256k1;
    use bitcoin_hashes::sha256d;
    use bitcoin;
    use bitcoin::network::constants::Network;
    use crate::outpoint::RgbOutPoint;
    use bitcoin::util::psbt::serialize::Serialize;
    use bitcoin::consensus::encode::Encodable;
    use bitcoin::consensus::serialize;

    const GENESIS_PUBKEY: bitcoin::PublicKey = bitcoin::PublicKey(true, secp256k1::PublicKey::from_str(
        "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f\
        4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5").unwrap());

    #[test]
    fn encode_utxo_outpoint_test() {
        let address = bitcoin::Address::p2wpkh(&GENESIS_PUBKEY, Network::Mainnet);
        let vout: [u8; 4] = [0, 0, 0, 0];

        let mut preimage = address.to_string().into_bytes();
        preimage.extend_from_slice(&vout);

        let mut engine = sha256d::Hash::engine();
        engine.write_all(&preimage.as_slice());

        let hash = sha256d::Hash::from_engine(engine);
        let outpoint = RgbOutPoint::UTXO(hash);

        let data = serialize(&outpoint);
        print!("{}", data);
    }

    #[test]
    fn encode_vout_outpoint_test() {

    }

    #[test]
    fn decode_utxo_outpoint_test() {

    }

    #[test]
    fn decode_utxo_outpoint_misformat_test() {

    }

    #[test]
    fn decode_vout_outpoint_test() {

    }

    #[test]
    fn decode_vout_outpoint_misformat_test() {

    }

    #[test]
    fn decode_rogue_outpoint_test() {

    }
}
