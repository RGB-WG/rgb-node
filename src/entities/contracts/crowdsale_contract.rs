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
use entities::contracts::data::ContractData::CrowdsaleData;
use bitcoin::util::address::Address;
use util::error::Error::NetworkMismatchError;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct CrowdsaleContractData {
    pub deposit_address: Address,
    pub price_sat: u32,
    pub from_block: u32,
    pub to_block: u32,
}

impl CrowdsaleContractData {
    pub fn new_data(deposit_address: Address, price_sat: u32, from_block: u32, to_block: u32) -> CrowdsaleContractData {
        CrowdsaleContractData {
            deposit_address,
            price_sat,
            from_block,
            to_block,
        }
    }

    pub fn new(title: String, description: String, contract_url: String, issuance_utxo: TxOutRef, network: Network, total_supply: u32, min_amount: u32, max_hops: u32, deposit_address: Address, price_sat: u32, from_block: u32, to_block: u32) -> Contract {
        Contract::new(
            ContractHeader::new(title, description, contract_url, issuance_utxo, network, total_supply, min_amount, max_hops),
            CrowdsaleData(CrowdsaleContractData::new_data(deposit_address, price_sat, from_block, to_block)),
        )
    }
}

impl VerifyContractData for CrowdsaleContractData {
    fn verify(&self, _utxos_spent_in: &HashMap<&TxOutRef, Transaction>, header: &ContractHeader) -> Result<(), util::error::Error> {
        if header.network != self.deposit_address.network {
            return Err(NetworkMismatchError(self.deposit_address.network));
        }

        Ok(())
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for CrowdsaleContractData {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Add the kind of contract
        2u8.consensus_encode(s)?;

        // Encode the `deposit_address` as address string
        self.deposit_address.to_string().consensus_encode(s)?;

        self.price_sat.consensus_encode(s)?;
        self.from_block.consensus_encode(s)?;
        self.to_block.consensus_encode(s)?;

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for CrowdsaleContractData {
    fn consensus_decode(d: &mut D) -> Result<CrowdsaleContractData, D::Error> {
        let address_string: String = ConsensusDecodable::consensus_decode(d)?;
        let deposit_address = Address::from_str(address_string.as_str());

        if let Err(err) = deposit_address {
            return Err(d.error(format!("Invalid deposit address `{}`: {:?}", address_string, err)));
        }

        Ok(CrowdsaleContractData {
            deposit_address: deposit_address.unwrap(),
            price_sat: ConsensusDecodable::consensus_decode(d)?,
            from_block: ConsensusDecodable::consensus_decode(d)?,
            to_block: ConsensusDecodable::consensus_decode(d)?,
        })
    }
}

impl<S: SimpleEncoder> Hashable<S> for CrowdsaleContractData {
    fn hashable_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Add the kind of contract
        2u8.consensus_encode(s)?;

        // Encode the `deposit_address` as address string
        self.deposit_address.to_string().consensus_encode(s)?;

        self.price_sat.consensus_encode(s)?;
        self.from_block.consensus_encode(s)?;
        self.to_block.consensus_encode(s)?;

        Ok(())
    }
}