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

//! RGB transaction output types
//!
//! Implementation of data structures used in RGB contracts and transaction proofs for
//! specifying particular  outputs and bindings to the on-chain transactions

use bitcoin_hashes::sha256d;
use bitcoin::consensus::encode::*;

use crate::*;

/// Outpoint for an RGB transaction, defined by the
/// [RGB Specification](https://github.com/rgb-org/spec/blob/master/01-rgb.md#rgboutpoint).
/// Can be of two different type, represented by the corresponding enum variants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RgbOutPoint {
    /// UTXO-based RGB transaction, pointing to the hash of some pre-existing UTXO
    /// with some `vout` in it
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
                (0x1 as u8).consensus_encode(s)?;
                // next we put the hash of the concatenated TX hash and 32-bit vout:
                // SHA256D(TX_HASH || OUTPUT_INDEX_AS_U32)
                hash.consensus_encode(s)
            },
            RgbOutPoint::Vout(vout) => {
                // 0x2 stands for address-based RGB transaction
                (0x2 as u8).consensus_encode(s)?;
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
        let byte: u8 = Decodable::consensus_decode(d)?;
        match byte {
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

/// RGB transaction details for each of the transaction outputs for assets transfer.
/// Triplets specifying type of the asset transferred, amount and output points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbOutEntry {
    /// Asset type (hash of the consensus-serialized asset issue contract)
    // TODO: Probably unnecessary due to #72 <https://github.com/rgb-org/spec/issues/72>
    pub asset_id: IdentityHash,
    /// Amount, 64-bytes (for compatibility with bitcoin amounts)
    pub amount: u64,
    /// Output point for the transfer
    pub out_point: RgbOutPoint
}

impl<S: Encoder> Encodable<S> for RgbOutEntry {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.asset_id.consensus_encode(s)?;
        self.amount.consensus_encode(s)?;
        self.out_point.consensus_encode(s)
    }
}

impl<D: Decoder> Decodable<D> for RgbOutEntry {
    fn consensus_decode(d: &mut D) -> Result<RgbOutEntry, Error> {
        let asset_id: IdentityHash = Decodable::consensus_decode(d)?;
        let amount: u64 = Decodable::consensus_decode(d)?;
        let out_point: RgbOutPoint = Decodable::consensus_decode(d)?;
        Ok(RgbOutEntry { asset_id, amount, out_point })
    }
}

/// Structure representing bitcoin transaction with a set of coloured outputs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbTransaction {
    /// Hash of the transaction as its identifier
    pub txid: sha256d::Hash,

    /// Set of coloured outputs
    pub vouts: Vec<u32>,
}

impl<S: Encoder> Encodable<S> for RgbTransaction {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.txid.consensus_encode(s)?;
        self.vouts.consensus_encode(s)
    }
}

impl<D: Decoder> Decodable<D> for RgbTransaction {
    fn consensus_decode(d: &mut D) -> Result<RgbTransaction, Error> {
        Ok(RgbTransaction {
            txid: Decodable::consensus_decode(d)?,
            vouts: Decodable::consensus_decode(d)?,
        })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use std::io::Write;

    use bitcoin;
    use bitcoin_hashes::{sha256d, Hash};
    use bitcoin::network::constants::Network;
    use crate::outputs::{RgbOutPoint, RgbOutEntry};
    use bitcoin::consensus::{serialize, deserialize};
    use crate::constants::IdentityHash;

    const GENESIS_PUBKEY: &str =
        "043f80a276a6550f68e360907aea2359120e4c358d904aef351dd6a478f4cbd74550b96215e243c\
        75a9f630b80f29d6013f2029059ef7330543e2444ae9e9af2a6";

    fn generate_pubkey() -> bitcoin::PublicKey {
        bitcoin::PublicKey::from_str(GENESIS_PUBKEY).unwrap()
    }

    fn generate_utxo_outpoint() -> RgbOutPoint {
        let address = bitcoin::Address::p2wpkh(&generate_pubkey(), Network::Bitcoin);
        let vout: [u8; 4] = [0, 0, 0, 0];

        let mut preimage = address.to_string().into_bytes();
        preimage.extend_from_slice(&vout);

        let mut engine = sha256d::Hash::engine();
        engine.write_all(&preimage.as_slice()).unwrap();

        let hash = sha256d::Hash::from_engine(engine);
        RgbOutPoint::UTXO(hash)
    }

    fn generate_asset_id() -> IdentityHash {
        let mut engine = sha256d::Hash::engine();
        engine.write_all(&[1, 2, 3, 4]).unwrap();
        sha256d::Hash::from_engine(engine)
    }

    fn generate_vout_outpoint() -> RgbOutPoint {
        RgbOutPoint::Vout(3)
    }

    #[test]
    fn encode_utxo_outpoint_test() {
        let outpoint = generate_utxo_outpoint();
        let data = serialize(&outpoint);
        let data_standard: [u8; 33] = [
            1, 181, 12, 89, 229, 61, 22, 250, 116, 213, 204, 253, 200, 43, 222, 217, 74, 18, 135,
            54, 11, 227, 50, 48, 90, 182, 117, 141, 145, 138, 111, 124, 23];
        assert_eq!(data[0], 0x1, "First byte of serialized UTXO-based outpoint must be equal to 0x1");
        assert_eq!(data.len(), data_standard.len(), "Wrong length of serialized data");
        assert_eq!(data[..], data_standard[..], "Broken serialization");
    }

    #[test]
    fn encode_vout_outpoint_test() {
        let outpoint = generate_vout_outpoint();
        let data = serialize(&outpoint);
        let data_standard: [u8; 5] = [2, 3, 0, 0, 0];
        assert_eq!(data[0], 0x2, "First byte of serialized vout-based outpoint must be equal to 0x1");
        assert_eq!(data.len(), data_standard.len(), "Wrong length of serialized data");
        assert_eq!(data[..], data_standard[..], "Broken serialization");
    }

    #[test]
    fn decode_utxo_outpoint_test() {
        let data_standard: [u8; 33] = [
            1, 181, 12, 89, 229, 61, 22, 250, 116, 213, 204, 253, 200, 43, 222, 217, 74, 18, 135,
            54, 11, 227, 50, 48, 90, 182, 117, 141, 145, 138, 111, 124, 23];
        let outpoint: RgbOutPoint = deserialize(&data_standard).unwrap();
        assert_eq!(outpoint, generate_utxo_outpoint());
    }

    #[test]
    fn decode_utxo_outpoint_different_test() {
        let data_standard: [u8; 33] = [
            1, 18, 12, 89, 229, 61, 22, 250, 116, 213, 204, 253, 200, 43, 222, 217, 74, 18, 135,
            54, 11, 227, 50, 48, 90, 182, 117, 141, 145, 138, 111, 124, 23];
        let outpoint: RgbOutPoint = deserialize(&data_standard).unwrap();
        assert_ne!(outpoint, generate_utxo_outpoint());
    }

    #[test]
    #[should_panic]
    fn decode_utxo_outpoint_misformat_test() {
        let data_standard: [u8; 33] = [
            3, 181, 12, 89, 229, 61, 22, 250, 116, 213, 204, 253, 200, 43, 222, 217, 74, 18, 135,
            54, 11, 227, 50, 48, 90, 182, 117, 141, 145, 138, 111, 124, 23];
        let outpoint: RgbOutPoint = deserialize(&data_standard).unwrap();
        assert_ne!(outpoint, generate_utxo_outpoint());
    }

    #[test]
    fn decode_vout_outpoint_test() {
        let data_standard: [u8; 5] = [2, 3, 0, 0, 0];
        let outpoint: RgbOutPoint = deserialize(&data_standard).unwrap();
        assert_eq!(outpoint, generate_vout_outpoint());
    }

    #[test]
    #[should_panic]
    fn decode_vout_outpoint_misformat_test() {
        let data_standard: [u8; 5] = [1, 3, 0, 0, 0];
        let outpoint: RgbOutPoint = deserialize(&data_standard).unwrap();
        assert_ne!(outpoint, generate_vout_outpoint());
    }

    #[test]
    #[should_panic]
    fn decode_rogue_outpoint_test() {
        let data_standard: [u8; 10] = [2, 4, 6, 7, 8, 3, 5, 6, 8, 9];
        let _: RgbOutPoint = deserialize(&data_standard).unwrap();
    }

    #[test]
    fn transcode_simple_outentry() {
        let outpoint = generate_vout_outpoint();
        let outentry = RgbOutEntry {
            asset_id: generate_asset_id(),
            amount: 13,
            out_point: outpoint
        };
        let data = serialize(&outentry);
        let outentry_transcoded: RgbOutEntry = deserialize(&data[..]).unwrap();
        assert_eq!(outentry, outentry_transcoded);
    }
}
