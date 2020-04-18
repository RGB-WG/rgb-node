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


use std::{str::FromStr, collections::HashMap};
use chrono::NaiveDateTime;
use lnpbp::{bp, bitcoin, bitcoin::secp256k1, rgb::*};
use lnpbp::bitcoin::{OutPoint};
use lnpbp::miniscript::Miniscript;

use super::{Amount, Error, Invoice, selection};

// Temporary types
type HistoryGraph = ();
type Signature = ();


#[derive(Clone, Debug, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub enum Supply {
    Unknown,
    PartiallyKnown(Amount),
    Known(Amount)
}

#[derive(Clone, Debug, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub struct Stock {
    pub ticker: String,
    pub title: String,
    pub description: Option<String>,
    pub total_supply: Option<Amount>,
    pub dust_limit: Amount,
    pub fractions: u8,
    pub signature: Signature,

    pub primary_issue: Issue,
    pub allocations: Vec<Allocation>,
}

impl Stock {
    pub fn init(genesis: Transition) -> Result<Self, Error> { unimplemented!() }
    pub fn extend(&mut self, history: HistoryGraph, payment: Option<Payment>) -> Result<Vec<Allocation>, Error> { unimplemented!() }

    #[cfg(feature="fa_issue")]
    pub fn issue(network: Network, ticker: &str, name: &str, descr: Option<&str>,
                 balances: HashMap<OutPoint, Amount>, precision: u8,
                 supply: Option<Uint256>, dust: Option<Uint256>) -> Result<Self, Error> { unimplemented!() }
    #[cfg(feature="fa_issue")]
    pub fn inflate(&mut self, ) -> Result<Self, Error> { unimplemented!() }
    pub fn transfer(&mut self, balances: HashMap<OutPoint, Amount>) -> Result<Transition, Error> { unimplemented!() }

    pub fn get_total_supply(&self) -> Supply { unimplemented!() }
    pub fn get_issued_supply(&self) -> Supply { unimplemented!() }
    pub fn is_issuance_completed(&self) -> bool { unimplemented!() }
    pub fn issues_iter(&self) -> IssueIter { unimplemented!() }

    pub fn total_holdings(&self) -> Amount { unimplemented!() }
    pub fn allocations_matching(&self, amount: Amount, strategy: &dyn selection::Strategy) -> Vec<Allocation> { unimplemented!() }
}

pub struct IssueIter {}
impl Iterator for IssueIter {
    type Item = Issue;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub enum NextIssuance {
    Prohibited,
    Unknown,
    Known(Box<Issue>)
}

#[derive(Clone, Debug, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub struct Issue {
    pub supply: Amount,
    pub next: NextIssuance,
}

#[derive(Clone, Debug, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub struct Allocation {
    pub amount: Amount,
    pub seal: OutPoint,
    pub payment: Option<Payment>,
}

impl FromStr for Allocation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

impl Allocation {
}


#[derive(Clone, Debug, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub enum Payer {
    BitcoinPubkey(bitcoin::PublicKey),
    BitcoinMultisig(Vec<bitcoin::PublicKey>, u8),
    BitcoinScript(Miniscript<bitcoin::PublicKey>),
    Tapscript(Miniscript<bitcoin::PublicKey>),
    LightningNode(secp256k1::PublicKey),
}

#[derive(Clone, Debug, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub struct Payment {
    pub date: NaiveDateTime,
    pub payer: Payer,
}

impl Payment {
}
