use bitcoin::BitcoinHash;
use bitcoin::blockdata::opcodes::*;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::script::Script;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize;
use bitcoin::network::serialize::serialize;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::Transaction;
use bitcoin::util::hash::Sha256dHash;
use std::collections::HashMap;
use super::bitcoin::network::constants::Network;
use super::bitcoin::OutPoint;
use super::traits::Verify;
use traits::NeededTx;

#[derive(Clone, Debug)]
pub struct Contract {
    pub title: String,
    /// Will be spent in the contract transaction
    pub issuance_utxo: OutPoint,
    /// Will own the issued assets
    pub initial_owner_utxo: OutPoint,
    pub network: Network,
    pub total_supply: u32,
}

impl Contract {
    pub fn get_asset_id(&self) -> Sha256dHash {
        self.bitcoin_hash()
    }
}

impl BitcoinHash for Contract {
    fn bitcoin_hash(&self) -> Sha256dHash { // all the fields
        Sha256dHash::from_data(&serialize(self).unwrap())
    }
}

impl Verify for Contract {
    fn get_needed_txs(&self) -> Vec<NeededTx> {
        vec![NeededTx::WhichSpendsOutPoint(self.issuance_utxo)]
    }

    fn verify(&self, needed_txs: &HashMap<&NeededTx, Transaction>) -> bool {
        let committing_tx = needed_txs.get(&NeededTx::WhichSpendsOutPoint(self.issuance_utxo)).unwrap();
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
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for Contract {
    fn consensus_encode(&self, s: &mut S) -> Result<(), serialize::Error> {
        self.title.consensus_encode(s)?;
        self.issuance_utxo.consensus_encode(s)?;
        self.initial_owner_utxo.consensus_encode(s)?;

        self.network.consensus_encode(s)?;
        self.total_supply.consensus_encode(s)
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for Contract {
    fn consensus_decode(d: &mut D) -> Result<Contract, serialize::Error> {
        let title: String = ConsensusDecodable::consensus_decode(d)?;
        let issuance_utxo: OutPoint = ConsensusDecodable::consensus_decode(d)?;
        let initial_owner_utxo: OutPoint = ConsensusDecodable::consensus_decode(d)?;

        Ok(Contract {
            title,
            issuance_utxo,
            initial_owner_utxo,
            network: ConsensusDecodable::consensus_decode(d)?,
            total_supply: ConsensusDecodable::consensus_decode(d)?,
        })
    }
}