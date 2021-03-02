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

use std::collections::{BTreeMap, BTreeSet};

use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use bitcoin::OutPoint;
use lnpbp::seals::OutpointReveal;
use rgb::{Consignment, ContractId, NodeId, SealEndpoint, Transition};

#[derive(Clone, Debug, Display, Api)]
#[api(encoding = "strict")]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display(inner)]
#[non_exhaustive]
pub enum Request {
    #[api(type = 0x0101)]
    #[display("add_schema({0})")]
    AddSchema(::rgb::Schema),

    #[api(type = 0x0103)]
    #[display("list_schemata()")]
    ListSchemata(),

    #[api(type = 0x0105)]
    #[display("read_schema({0})")]
    ReadSchema(::rgb::SchemaId),

    #[api(type = 0x0201)]
    #[display("add_genesis({0})")]
    AddGenesis(::rgb::Genesis),

    #[api(type = 0x0203)]
    #[display("list_geneses()")]
    ListGeneses(),

    #[api(type = 0x0205)]
    #[display("read_genesis({0})")]
    ReadGenesis(::rgb::ContractId),

    #[api(type = 0x0301)]
    #[display("read_transitions(...)")]
    ReadTransitions(Vec<::rgb::NodeId>),

    #[api(type = 0x0401)]
    Consign(crate::rpc::stash::ConsignRequest),

    #[api(type = 0x0403)]
    Validate(::rgb::Consignment),

    #[api(type = 0x0405)]
    Merge(crate::rpc::stash::MergeRequest),

    #[api(type = 0x0407)]
    #[display("forget(...)")]
    Forget(Vec<(::rgb::NodeId, u16)>),
}

#[derive(Clone, StrictEncode, StrictDecode, Debug, Display)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display("consign({contract_id}, ...)")]
pub struct ConsignRequest {
    pub contract_id: ContractId,
    pub inputs: BTreeSet<OutPoint>,
    pub transition: Transition,
    pub other_transition_ids: BTreeMap<ContractId, NodeId>,
    pub endpoints: BTreeSet<SealEndpoint>,
    pub psbt: Psbt,
}

#[derive(Clone, StrictEncode, StrictDecode, Debug, Display)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display("merge(...)")]
pub struct MergeRequest {
    pub consignment: Consignment,
    pub reveal_outpoints: Vec<OutpointReveal>,
}
