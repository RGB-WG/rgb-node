use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::network::constants::Network;
use entities::contracts::Contract;
use entities::rgb_output::RgbOutPoint;
use entities::traits::VerifyContractData;
use std::collections::HashMap;
use util;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::util::hash::Sha256dHash;
use entities::contracts::header::ContractHeader;
use entities::contracts::data::ContractData::EmissionData;
use entities::traits::Hashable;

#[derive(Clone, Debug)]
pub struct EmissionContractData {
    pub owner_utxo: RgbOutPoint
}

impl EmissionContractData {
    pub fn new_data(owner_utxo: RgbOutPoint) -> EmissionContractData {
        EmissionContractData {
            owner_utxo
        }
    }

    pub fn new(title: String, description: String, contract_url: String, issuance_utxo: TxOutRef, network: Network, total_supply: u32, min_amount: u32, max_hops: u32, owner_utxo: RgbOutPoint) -> Contract {
        Contract::new(
            ContractHeader::new(title, description, contract_url, issuance_utxo, network, total_supply, min_amount, max_hops),
            EmissionData(EmissionContractData::new_data(owner_utxo)),
        )
    }
}

impl VerifyContractData for EmissionContractData {
    fn verify(&self, _utxos_spent_in: &HashMap<&TxOutRef, Transaction>, _header: &ContractHeader) -> Result<(), util::error::Error> {
        Ok(())
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for EmissionContractData {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Add the kind of contract
        1u8.consensus_encode(s)?;

        // Encode the `owner_utxo`
        self.owner_utxo.consensus_encode(s)?;

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for EmissionContractData {
    fn consensus_decode(d: &mut D) -> Result<EmissionContractData, D::Error> {
        Ok(EmissionContractData {
            owner_utxo: ConsensusDecodable::consensus_decode(d)?
        })
    }
}

impl<S: SimpleEncoder> Hashable<S> for EmissionContractData {
    fn hashable_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Add the kind of contract
        1u8.consensus_encode(s)?;

        // Encode the `owner_utxo`
        self.owner_utxo.consensus_encode(s)?;

        Ok(())
    }
}