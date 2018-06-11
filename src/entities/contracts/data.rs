use entities::contracts::emission_contract::EmissionContractData;
use entities::traits::VerifyContractData;
use std::collections::HashMap;
use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::blockdata::transaction::Transaction;
use entities::contracts::header::ContractHeader;
use entities::contracts::data::ContractData::EmissionData;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::network::serialize::SimpleDecoder;
use util;
use entities::traits::Hashable;
use entities::contracts::crowdsale_contract::CrowdsaleContractData;
use entities::contracts::data::ContractData::CrowdsaleData;

#[derive(Clone, Debug)]
pub enum ContractData {
    EmissionData(EmissionContractData),
    CrowdsaleData(CrowdsaleContractData),
}

impl VerifyContractData for ContractData {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>, header: &ContractHeader) -> Result<(), util::error::Error> {
        match *self {
            EmissionData(ref contract) => contract.verify(utxos_spent_in, header),
            CrowdsaleData(ref contract) => contract.verify(utxos_spent_in, header)
        }
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for ContractData {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            EmissionData(ref contract) => contract.consensus_encode(s),
            CrowdsaleData(ref contract) => contract.consensus_encode(s),
        }
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for ContractData {
    fn consensus_decode(d: &mut D) -> Result<ContractData, D::Error> {
        let contract_data_type: u8 = ConsensusDecodable::consensus_decode(d)?;

        match contract_data_type {
            0x01 => {
                let contract_data: EmissionContractData = ConsensusDecodable::consensus_decode(d)?;
                Ok(ContractData::EmissionData(contract_data))
            }
            0x02 => {
                let contract_data: CrowdsaleContractData = ConsensusDecodable::consensus_decode(d)?;
                Ok(ContractData::CrowdsaleData(contract_data))
            }
            x => Err(d.error(format!("ContractData type {:02x} not understood", x)))
        }
    }
}

impl<S: SimpleEncoder> Hashable<S> for ContractData {
    fn hashable_encode(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            EmissionData(ref contract) => contract.hashable_encode(s),
            CrowdsaleData(ref contract) => contract.hashable_encode(s),
        }
    }
}