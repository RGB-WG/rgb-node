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

use lnpbp::rgb::prelude::*;

use crate::error::InteroperableError;

pub trait Store {
    fn schema_ids(&self) -> Result<Vec<SchemaId>, InteroperableError>;
    fn schema(&self, id: SchemaId) -> Result<Schema, InteroperableError>;
    fn has_schema(&self, id: SchemaId) -> Result<bool, InteroperableError>;
    fn add_schema(&self, schema: &Schema) -> Result<bool, InteroperableError>;
    fn remove_schema(&self, id: SchemaId) -> Result<bool, InteroperableError>;

    fn contract_ids(&self) -> Result<Vec<ContractId>, InteroperableError>;
    fn genesis(&self, id: ContractId) -> Result<Genesis, InteroperableError>;
    fn has_genesis(&self, id: ContractId) -> Result<bool, InteroperableError>;
    fn add_genesis(&self, genesis: &Genesis) -> Result<bool, InteroperableError>;
    fn remove_genesis(&self, id: ContractId) -> Result<bool, InteroperableError>;
}
