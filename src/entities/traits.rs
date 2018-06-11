use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use std::collections::{HashMap, HashSet};
use entities::proofs::header::ProofHeader;
use entities::contracts::header::ContractHeader;
use bitcoin::network::serialize::SimpleEncoder;
use util;

pub trait Verify {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>) -> Result<(), util::error::Error>;
    fn get_necessary_txs(&self, set: &mut HashSet<TxOutRef>);
}

pub trait VerifyData {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>, header: &ProofHeader) -> Result<(), util::error::Error>;
}

pub trait VerifyContractData {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>, header: &ContractHeader) -> Result<(), util::error::Error>;
}

pub trait ContainsSignatures {
    fn push_signature(&mut self, signature: Vec<u8>, s2c_commitment: Vec<u8>);
}

pub trait Hashable<S: SimpleEncoder> {
    fn hashable_encode(&self, s: &mut S) -> Result<(), S::Error>;
}