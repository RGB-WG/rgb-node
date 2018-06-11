use bitcoin::blockdata::transaction::TxOutRef;
use entities::traits::Verify;
use std::collections::HashMap;
use bitcoin::blockdata::transaction::Transaction;
use util::error::Error::MissingTx;
use std::collections::HashSet;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::util::hash::Sha256dHash;
use entities::traits::ContainsSignatures;
use bitcoin::network::constants::Network;
use util;
use entities::traits::Hashable;

#[derive(Clone, Debug)]
pub struct ContractHeader {
    pub title: String,
    pub description: String,
    //pub version: u16,
    pub contract_url: String,
    pub issuance_utxo: TxOutRef,
    //pub next_issuance_enabled: bool,
    //pub next_issuance_utxo: Option<TxOutRef>,
    pub network: Network,
    pub total_supply: u32,
    pub min_amount: u32,
    pub max_hops: u32,
    pub signatures: Vec<Vec<u8>>,
    // TODO: look for a struct in rust-secp256k1
    pub tx_commitment_r: Vec<Vec<u8>>, // TODO: once again, look for a struct in rust-secp256k1
}

impl ContractHeader {
    pub fn new(title: String, description: String, contract_url: String, issuance_utxo: TxOutRef, network: Network, total_supply: u32, min_amount: u32, max_hops: u32) -> ContractHeader {
        ContractHeader {
            title,
            description,
            contract_url,
            issuance_utxo,
            network,
            total_supply,
            min_amount,
            max_hops,
            signatures: Vec::new(),
            tx_commitment_r: Vec::new(),
        }
    }
}

impl ContainsSignatures for ContractHeader {
    fn push_signature(&mut self, signature: Vec<u8>, s2c_commitment: Vec<u8>) {
        self.signatures.push(signature);
        self.tx_commitment_r.push(s2c_commitment);
    }
}

impl Verify for ContractHeader {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>) -> Result<(), util::error::Error> {
        // TODO: verify the commitment to this contract

        let tx_committing_to_this = utxos_spent_in.get(&self.issuance_utxo);

        if tx_committing_to_this == None {
            return Err(MissingTx(self.issuance_utxo));
        }

        Ok(())
    }

    fn get_necessary_txs(&self, set: &mut HashSet<TxOutRef>) {
        set.insert(self.issuance_utxo);
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for ContractHeader {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Encode the version (16 bit so that we can use some bits as switches if necessary in the future)
        (1u16).consensus_encode(s)?;

        self.title.consensus_encode(s)?;
        self.description.consensus_encode(s)?;
        self.contract_url.consensus_encode(s)?;

        // Encode the `issuance_utxo` (TxOutRef)
        self.issuance_utxo.txid.consensus_encode(s)?;
        (self.issuance_utxo.index as u32).consensus_encode(s)?; // TODO: Is the casting here really necessary?

        self.network.consensus_encode(s)?;

        self.total_supply.consensus_encode(s)?;
        self.min_amount.consensus_encode(s)?;
        self.max_hops.consensus_encode(s)?;

        // Encode the signatures and the R used to verify the commitments.
        self.signatures.consensus_encode(s)?;
        self.tx_commitment_r.consensus_encode(s)?;

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for ContractHeader {
    fn consensus_decode(d: &mut D) -> Result<ContractHeader, D::Error> {
        let _version: u16 = ConsensusDecodable::consensus_decode(d)?;

        let title: String = ConsensusDecodable::consensus_decode(d)?;
        let description: String = ConsensusDecodable::consensus_decode(d)?;
        let contract_url: String = ConsensusDecodable::consensus_decode(d)?;

        let issuance_utxo_txid: Sha256dHash = ConsensusDecodable::consensus_decode(d)?;
        let issuance_utxo_index: u32 = ConsensusDecodable::consensus_decode(d)?;

        let network: Network = ConsensusDecodable::consensus_decode(d)?;

        let total_supply: u32 = ConsensusDecodable::consensus_decode(d)?;
        let min_amount: u32 = ConsensusDecodable::consensus_decode(d)?;
        let max_hops: u32 = ConsensusDecodable::consensus_decode(d)?;

        let signatures: Vec<Vec<u8>> = ConsensusDecodable::consensus_decode(d)?;
        let tx_commitment_r: Vec<Vec<u8>> = ConsensusDecodable::consensus_decode(d)?;

        Ok(ContractHeader {
            title,
            description,
            contract_url,
            issuance_utxo: TxOutRef {
                txid: issuance_utxo_txid,
                index: issuance_utxo_index as usize,
            },
            network,
            total_supply,
            min_amount,
            max_hops,
            signatures,
            tx_commitment_r,
        })
    }
}

impl<S: SimpleEncoder> Hashable<S> for ContractHeader {
    fn hashable_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Encode the version (16 bit so that we can use some bits as switches if necessary in the future)
        (1u16).consensus_encode(s)?;

        self.title.consensus_encode(s)?;
        self.description.consensus_encode(s)?;
        self.contract_url.consensus_encode(s)?;

        // Encode the `issuance_utxo` (TxOutRef)
        self.issuance_utxo.txid.consensus_encode(s)?;
        (self.issuance_utxo.index as u32).consensus_encode(s)?; // TODO: Is the casting here really necessary?

        self.network.consensus_encode(s)?;

        self.total_supply.consensus_encode(s)?;
        self.min_amount.consensus_encode(s)?;
        self.max_hops.consensus_encode(s)?;

        // NO SIGNATURES. TODO: do we need R(s)?

        Ok(())
    }
}