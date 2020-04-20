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


use std::collections::HashMap;

use lnpbp::rgb::ContractId;
use lnpbp::rgb::schemata::fungible::Balances;


pub trait DataProvider {
    type Error: std::error::Error;

    fn get_balances(&self, contract_id: ContractId) -> Result<Balances, Self::Error>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, From)]
#[display_from(Debug)]
pub enum Error {

}

pub struct Accountant {

}

impl Accountant {
    fn get_balances(&self, contract_id: ContractId) -> Result<Balances, Error> {
        Ok(HashMap::new())
    }
}
