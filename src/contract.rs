use bitcoin::{Address, OutPoint};
use bitcoin::consensus::encode::*;
use bitcoin::network::constants::Network;

#[derive(Clone, Debug)]
pub enum CommitmentScheme {
    OpReturn = 0, PayToContract = 1
}

#[derive(Clone, Debug)]
pub enum BlueprintType {
    Issue = 0x01, Crowdsale = 0x02, Reissue = 0x03,
}

// Contract header fields required by the specification
#[derive(Clone, Debug)]
pub struct ContractHeader {
    pub version: u16,
    pub title: String,
    pub description: Option<String>,
    pub contract_url: Option<String>,
    pub issuance_utxo: OutPoint,
    pub network: Network,
    pub total_supply: u64,
    pub min_amount: u64,
    pub max_hops: Option<u32>,
    pub reissuance_enabled: Bool,
    pub reissuance_utxo: Option<OutPoint>,
    pub burn_address: Option<Address>,
    pub commitment_scheme: CommitmentScheme,
    pub blueprint_type: BlueprintType,
}

pub trait ContractBody {

}

#[derive(Clone, Debug)]
pub struct Contract {
    pub header: ContractHeader,
    pub body: ContractBody,
    pub initial_owner_utxo: OutPoint,
}

impl Contract {

}

impl<S: Encoder> Encodable<S> for Contract {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {

    }
}

impl<D: Encoder> Decodable<D> for Contract {
    fn consensus_decode(d: &mut D) -> Result<Contract, Error> {
        let mut contract = Contract();
        Ok(contract)
    }
}
