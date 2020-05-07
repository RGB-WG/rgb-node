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

use crate::error::InteroperableError;
use crate::fungible::Asset;
use lnpbp::rgb::prelude::*;

pub trait Cache {
    fn assets(&self) -> Result<Vec<&Asset>, InteroperableError>;
    fn asset(&self, id: ContractId) -> Result<&Asset, InteroperableError>;
    fn has_asset(&self, id: ContractId) -> Result<bool, InteroperableError>;
    fn add_asset(&mut self, asset: Asset) -> Result<bool, InteroperableError>;
    fn remove_asset(&mut self, id: ContractId) -> Result<bool, InteroperableError>;
}
