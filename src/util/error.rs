use bitcoin;
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use entities::contracts::data::ContractData;
use bitcoin::network::constants::Network;

#[derive(Debug)]
pub enum Error {
    BitcoinError(bitcoin::util::Error),

    InputOutputMismatch,
    // TODO: additional data
    MissingTx(TxOutRef),
    InvalidOutputIndex(u16),
    EmissionProofHasInputs(usize),
    InvalidEmissionOwner(TxOutRef),
    LoopDetected(TxOutRef),
    WrongOutputSpentEntry(TxOutRef),

    WrongContractKind(ContractData),
    NetworkMismatchError(Network),

    NoSignatureFound(Transaction),
}