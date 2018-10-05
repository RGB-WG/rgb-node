use bitcoin::blockdata::script::Script;
use bitcoin::OutPoint;
use bitcoin::Transaction;
use bitcoin::util::hash::Sha256dHash;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum NeededTx {
    FromTXID(Sha256dHash),
    WhichSpendsOutPoint(OutPoint),
}

pub trait Verify {
    fn get_needed_txs(&self) -> Vec<NeededTx>;
    fn verify(&self, needed_txs: &HashMap<&NeededTx, Transaction>) -> bool;
    fn get_expected_script(&self) -> Script;
}