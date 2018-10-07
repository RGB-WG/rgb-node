use bitcoin::BitcoinHash;
use bitcoin::blockdata::opcodes::*;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::script::Script;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize;
use bitcoin::network::serialize::RawEncoder;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::Transaction;
use bitcoin::util::address::Address;
use bitcoin::util::hash::Sha256dHash;
use std::collections::HashMap;
use std::str::FromStr;
use super::bitcoin::network::constants::Network;
use super::bitcoin::OutPoint;
use super::traits::Verify;
use traits::NeededTx;

#[derive(Clone, Debug)]
pub struct Contract {
    pub version: u16,
    pub title: String,
    pub issuance_utxo: OutPoint,
    pub initial_owner_utxo: OutPoint,
    pub burn_address: Address,
    pub network: Network,
    pub total_supply: u32,
    pub tx_committing_to_this: Option<Sha256dHash>,
}

impl Contract {
    pub fn get_asset_id(&self) -> Sha256dHash {
        self.bitcoin_hash()
    }
}

impl BitcoinHash for Contract {
    fn bitcoin_hash(&self) -> Sha256dHash {
        // skip tx_committing_to_this, not relevant for consensus

        let encoded: Vec<u8> = Vec::new();
        let mut enc = RawEncoder::new(encoded);

        self.version.consensus_encode(&mut enc).unwrap();
        self.title.consensus_encode(&mut enc).unwrap();
        self.issuance_utxo.consensus_encode(&mut enc).unwrap();
        self.initial_owner_utxo.consensus_encode(&mut enc).unwrap();
        self.burn_address.to_string().consensus_encode(&mut enc).unwrap();
        self.network.consensus_encode(&mut enc).unwrap();
        self.total_supply.consensus_encode(&mut enc).unwrap();

        enc.into_inner().bitcoin_hash()
    }
}

impl Verify for Contract {
    fn get_needed_txs(&self) -> Vec<NeededTx> {
        vec![NeededTx::WhichSpendsOutPoint(self.issuance_utxo)]
    }

    fn verify(&self, needed_txs: &HashMap<&NeededTx, Transaction>) -> bool {
        let committing_tx = self.get_tx_committing_to_self(needed_txs).unwrap();
        let expected = self.get_expected_script();

        // Check the outputs
        let mut found_output = false;
        for i in 0..committing_tx.output.len() {
            found_output = found_output || committing_tx.output[i].script_pubkey == expected;
        }

        if !found_output {
            println!("invalid commitment");
            return false;
        }

        true
    }

    fn get_expected_script(&self) -> Script {
        let burn_script_builder = Builder::new();

        let burn_script_builder = burn_script_builder.push_opcode(All::OP_RETURN);
        let burn_script_builder = burn_script_builder.push_slice(self.bitcoin_hash().as_bytes());

        burn_script_builder.into_script()
    }

    fn get_tx_committing_to_self<'m>(&self, needed_txs: &'m HashMap<&NeededTx, Transaction>) -> Option<&'m Transaction> {
        match self.tx_committing_to_this {
            Some(txid) => needed_txs.get(&NeededTx::FromTXID(txid)), // either by using the hint in the contract
            None => needed_txs.get(&NeededTx::WhichSpendsOutPoint(self.issuance_utxo)) // or get the tx which spends the issuance_utxo
        }
    }

    fn set_tx_committing_to_self(&mut self, tx: &Transaction) {
        self.tx_committing_to_this = Some(tx.txid());
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for Contract {
    fn consensus_encode(&self, s: &mut S) -> Result<(), serialize::Error> {
        self.version.consensus_encode(s)?;
        self.title.consensus_encode(s)?;
        self.issuance_utxo.consensus_encode(s)?;
        self.initial_owner_utxo.consensus_encode(s)?;
        self.burn_address.to_string().consensus_encode(s)?;
        self.network.consensus_encode(s)?;
        self.total_supply.consensus_encode(s)?;
        self.tx_committing_to_this.consensus_encode(s)
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for Contract {
    fn consensus_decode(d: &mut D) -> Result<Contract, serialize::Error> {
        let version: u16 = ConsensusDecodable::consensus_decode(d)?;
        let title: String = ConsensusDecodable::consensus_decode(d)?;
        let issuance_utxo: OutPoint = ConsensusDecodable::consensus_decode(d)?;
        let initial_owner_utxo: OutPoint = ConsensusDecodable::consensus_decode(d)?;
        let burn_address_str: String = ConsensusDecodable::consensus_decode(d)?;

        Ok(Contract {
            version,
            title,
            issuance_utxo,
            initial_owner_utxo,
            burn_address: Address::from_str(burn_address_str.as_str()).unwrap(),
            network: ConsensusDecodable::consensus_decode(d)?,
            total_supply: ConsensusDecodable::consensus_decode(d)?,
            tx_committing_to_this: ConsensusDecodable::consensus_decode(d)?
        })
    }
}