// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::cell::RefCell;

use electrum_client::{client::ElectrumPlaintextStream, types::Error, Client};

use lnpbp::bitcoin::{Transaction, Txid};
use lnpbp::rgb::validation::{TxResolver, TxResolverError};

fn map_electrum_err(other: Error) -> TxResolverError {
    log::error!("Electrum error: {:?}", other);

    TxResolverError
}

pub struct ElectrumTxResolver {
    client: RefCell<Client<ElectrumPlaintextStream>>,
}

impl ElectrumTxResolver {
    pub fn new(server: &str) -> Result<Self, Error> {
        Ok(ElectrumTxResolver {
            client: RefCell::new(Client::new(server)?),
        })
    }
}

impl TxResolver for &ElectrumTxResolver {
    fn resolve(&self, txid: &Txid) -> Result<Option<(Transaction, u64)>, TxResolverError> {
        log::debug!("Resolving txid {}", txid);

        let tx = self
            .client
            .borrow_mut()
            .transaction_get(txid)
            .map_err(map_electrum_err)?;

        let input_amount = tx
            .input
            .iter()
            .map(|i| -> Result<_, Error> {
                Ok((
                    self.client
                        .borrow_mut()
                        .transaction_get(&i.previous_output.txid)?,
                    i.previous_output.vout,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()
            .map_err(map_electrum_err)?
            .into_iter()
            .map(|(tx, vout)| tx.output[vout as usize].value)
            .fold(0, |sum, i| i + sum);
        let output_amount = tx.output.iter().fold(0, |sum, o| sum + o.value);
        let fee = input_amount
            .checked_sub(output_amount)
            .ok_or(TxResolverError)?;

        log::debug!("Calculated fee: {}", fee);

        Ok(Some((tx, fee)))
    }
}
