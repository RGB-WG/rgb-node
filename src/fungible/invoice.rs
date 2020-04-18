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


use lnpbp::{bp, rgb};
use lnpbp::bitcoin::util::psbt::PartiallySignedTransaction;
use lightning_invoice::Invoice as LightningInvoice;
use lnpbp::common::internet::InetSocketAddr;

use crate::marker;


/// API endpoint for extra-transaction proof transfer
pub enum Endpoint {
    /// RGB server (non-anonymizing personal server or on-line wallet)
    RGB(InetSocketAddr),

    /// Bifrost server (anonymizing service)
    Bifrost(InetSocketAddr),

    /// Pay via Spectrum protocol using Lightning Network
    Spectrum(LightningInvoice)
}

/// Definition of where the transferred asset have to be assigned to
pub enum SealDefinition {
    /// Use existing UTXO (but blinded, so we don't know which is it)
    ExistingUtxo(bp::blind::OutpointHash),

    /// Create a new UTXO in the commitment transaction using this
    /// partially-signed transaction template and assign assets to a given
    /// output within it
    NewUtxo(PartiallySignedTransaction, u16)
}

/// Fungible invoice data
pub struct Invoice {
    pub contract_id: rgb::ContractId,
    pub assign_to: SealDefinition,
    pub amount: rgb::data::Amount,
    pub endpoints: Vec<Endpoint>,

    /// Requires sender to provide accessory information on commitment
    /// transaction ids
    pub provide_txids: bool,
}


/// Marker implementation
impl marker::Invoice for Invoice { }
