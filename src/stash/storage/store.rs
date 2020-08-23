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

use crate::error::ServiceErrorDomain;

pub trait Store {
    type Error: ::std::error::Error + Into<ServiceErrorDomain>;

    fn schema_ids(&self) -> Result<Vec<SchemaId>, Self::Error>;
    fn schema(&self, id: &SchemaId) -> Result<Schema, Self::Error>;
    fn has_schema(&self, id: &SchemaId) -> Result<bool, Self::Error>;
    fn add_schema(&self, schema: &Schema) -> Result<bool, Self::Error>;
    fn remove_schema(&self, id: &SchemaId) -> Result<bool, Self::Error>;

    fn contract_ids(&self) -> Result<Vec<ContractId>, Self::Error>;
    fn genesis(&self, id: &ContractId) -> Result<Genesis, Self::Error>;
    fn has_genesis(&self, id: &ContractId) -> Result<bool, Self::Error>;
    fn add_genesis(&self, genesis: &Genesis) -> Result<bool, Self::Error>;
    fn remove_genesis(&self, id: &ContractId) -> Result<bool, Self::Error>;

    fn anchor(&self, id: &AnchorId) -> Result<Anchor, Self::Error>;
    fn has_anchor(&self, id: &AnchorId) -> Result<bool, Self::Error>;
    fn add_anchor(&self, anchor: &Anchor) -> Result<bool, Self::Error>;
    fn remove_anchor(&self, id: &AnchorId) -> Result<bool, Self::Error>;

    fn transition(&self, id: &TransitionId) -> Result<Transition, Self::Error>;
    fn has_transition(&self, id: &TransitionId) -> Result<bool, Self::Error>;
    fn add_transition(&self, transition: &Transition) -> Result<bool, Self::Error>;
    fn remove_transition(&self, id: &TransitionId) -> Result<bool, Self::Error>;
}
