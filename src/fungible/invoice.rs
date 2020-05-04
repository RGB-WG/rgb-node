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


use std::{fmt, str::FromStr};
use regex::Regex;
use lightning_invoice::Invoice as LightningInvoice;

use lnpbp::{bp, rgb};
use lnpbp::bitcoin::util::psbt::PartiallySignedTransaction;
use lnpbp::common::internet::InetSocketAddr;

use crate::marker;
use lnpbp::miniscript::bitcoin::hashes::core::fmt::Formatter;


/// API endpoint for extra-transaction proof transfer
#[derive(Clone, Debug, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub enum Endpoint {
    /// RGB server (non-anonymizing personal server or on-line wallet)
    RGB(InetSocketAddr),

    /// Bifrost server (anonymizing service)
    Bifrost(InetSocketAddr),

    /// Pay via Spectrum protocol using Lightning Network
    Spectrum(LightningInvoice)
}

/// Definition of where the transferred asset have to be assigned to
#[derive(Clone, Debug, PartialEq, Display)]
#[display_from(Debug)]
pub enum SealDefinition {
    /// Use existing UTXO (but blinded, so we don't know which is it)
    ExistingUtxo(bp::blind::OutpointHash),

    /// Create a new UTXO in the commitment transaction using this
    /// partially-signed transaction template and assign assets to a given
    /// output within it
    NewUtxo(PartiallySignedTransaction, u16)
}

/// Fungible invoice data
#[derive(Clone, Debug, PartialEq, Display)]
#[display_from(Debug)]
pub struct Invoice {
    pub contract_id: rgb::ContractId,
    pub assign_to: SealDefinition,
    pub amount: rgb::data::Amount,
    pub endpoints: Vec<Endpoint>,

    /// Requires sender to provide accessory information on commitment
    /// transaction ids
    pub provide_txids: bool,
}

impl FromStr for Invoice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^rgb:([\w\d]{64}):(\d+)@()/(.*)$").expect("Regex parse failure");
        if let Some(m) = re.captures(&s.to_ascii_lowercase()) {
            match (m.get(1), m.get(2), m.get(3), m.get(4)) {
                (Some(id), Some(amount), Some(seal), _) => {
                    Ok(Self {
                        contract_id: id.as_str().parse()?,
                        assign_to: seal.as_str().parse()?,
                        amount: amount.as_str().parse()?,
                        endpoints: vec![],
                        provide_txids: false
                    })
                },
                _ => Err("Wrong invoice format".to_string()),
            }
        } else {
            Err("Wrong invoice format".to_string())
        }
    }
}

impl fmt::Display for Invoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "rgb:{}:{}@{}",
            self.contract_id,
            self.amount,
            self.assign_to
        )
    }
}


/// Marker implementation
impl marker::Invoice for Invoice { }
