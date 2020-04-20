// Kaleidoscope: RGB command-line wallet utility
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Alekos Filini <alekos.filini@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.


use std::fmt;
use std::collections::HashMap;
use lnpbp::bitcoin;
use bitcoin::{OutPoint, util::bip32};
use lnpbp::bp;
use lnpbp::rgb::data::Amount as AssetAmount;
use lnpbp::rgb::transition::TransitionId;
use lnpbp::miniscript::bitcoin::hashes::core::fmt::Formatter;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum SpendingStructure {
    P2PKH, P2WPKH
}

impl fmt::Display for SpendingStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", match self {
            SpendingStructure::P2PKH => "P2PKH",
            SpendingStructure::P2WPKH => "P2WPKH",
            _ => "Unsupported"
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DepositTerminal {
    pub outpoint: OutPoint,
    pub derivation_index: usize,
    pub spending_structure: SpendingStructure,
    pub bitcoins: bitcoin::Amount,
    pub fungibles: HashMap<TransitionId, Vec<(AssetAmount, TransitionId)>>
}

