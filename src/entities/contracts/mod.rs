use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::util::uint::Uint256;
use entities::contracts::ContractData::EmissionData;
use entities::contracts::emission_contract::EmissionContractData;
use entities::traits::ContainsSignatures;
use entities::traits::Verify;
use entities::traits::VerifyContractData;
use std::collections::{HashMap, HashSet};
use util;
use util::error::Error::MissingTx;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::util::hash::Sha256dHash;
use entities::contracts::header::ContractHeader;
use entities::contracts::data::ContractData;
use bitcoin::util::hash::Sha256dEncoder;
use entities::traits::Hashable;

pub mod header;
pub mod data;

pub mod emission_contract;
pub mod crowdsale_contract;

#[derive(Clone, Debug)]
pub struct Contract {
    pub header: ContractHeader,
    pub data: ContractData,
}

impl Contract {
    pub fn new(header: ContractHeader, data: ContractData) -> Contract {
        Contract {
            header,
            data,
        }
    }

    pub fn token_id(&self) -> Sha256dHash {
        let mut enc = Sha256dEncoder::new();
        self.hashable_encode(&mut enc).unwrap();
        enc.into_hash()
    }
}

impl Verify for Contract {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>) -> Result<(), util::error::Error> {
        Ok(())
            .and_then(|_: ()| { self.header.verify(utxos_spent_in) })
            .and_then(|_: ()| { self.data.verify(utxos_spent_in, &self.header) })
    }

    fn get_necessary_txs(&self, set: &mut HashSet<TxOutRef>) {
        self.header.get_necessary_txs(set);
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for Contract {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        self.header.consensus_encode(s)?;
        self.data.consensus_encode(s)?;

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for Contract {
    fn consensus_decode(d: &mut D) -> Result<Contract, D::Error> {
        Ok(Contract {
            header: ConsensusDecodable::consensus_decode(d)?,
            data: ConsensusDecodable::consensus_decode(d)?,
        })
    }
}

impl<S: SimpleEncoder> Hashable<S> for Contract {
    fn hashable_encode(&self, s: &mut S) -> Result<(), S::Error> {
        self.header.hashable_encode(s)?;
        self.data.hashable_encode(s)?;

        Ok(())
    }
}