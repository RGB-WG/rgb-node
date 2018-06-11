use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use util;
use util::error::Error::InvalidOutputIndex;

pub trait BuildFromTx {
    fn from_tx(tx: &Transaction, index: usize) -> Result<TxOutRef, util::error::Error>;
}

impl BuildFromTx for TxOutRef {
    fn from_tx(tx: &Transaction, index: usize) -> Result<TxOutRef, util::error::Error> {
        if index >= tx.output.len() {
            return Err(InvalidOutputIndex(index as u16));
        }

        Ok(TxOutRef {
            txid: tx.txid(),
            index,
        })
    }
}

pub trait VerifyInputs {
    fn is_input(&self, tx_out_ref: &TxOutRef) -> bool;
}

impl VerifyInputs for Transaction {
    fn is_input(&self, tx_out_ref: &TxOutRef) -> bool {
        self.input
            .iter()
            .any(|ref input| {
                input.prev_hash == tx_out_ref.txid && input.prev_index == tx_out_ref.index as u32
            })
    }
}