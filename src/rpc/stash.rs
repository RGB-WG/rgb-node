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
use bp::seals::OutpointReveal;
use rgb::{
    Consignment, ContractId, Disclosure, Genesis, NodeId, Schema, SchemaId,
    SealEndpoint, Transition,
};

#[derive(Clone, Debug, Display, Api)]
#[api(encoding = "strict")]
#[display(inner)]
#[non_exhaustive]
pub enum Request {
    #[api(type = 0x0101)]
    #[display("add_schema({0})")]
    AddSchema(Schema),

    #[api(type = 0x0103)]
    #[display("list_schemata()")]
    ListSchemata(),

    #[api(type = 0x0105)]
    #[display("read_schema({0})")]
    ReadSchema(SchemaId),

    #[api(type = 0x0201)]
    #[display("add_genesis({0})")]
    AddGenesis(Genesis),

    #[api(type = 0x0203)]
    #[display("list_geneses()")]
    ListGeneses(),

    #[api(type = 0x0205)]
    #[display("read_genesis({0})")]
    ReadGenesis(ContractId),

    #[api(type = 0x0301)]
    #[display("read_transitions(...)")]
    ReadTransitions(Vec<NodeId>),

    #[api(type = 0x0401)]
    Transfer(TransferRequest),

    #[api(type = 0x0403)]
    #[display("validate({0})")]
    Validate(Consignment),

    #[api(type = 0x0405)]
    Accept(AcceptRequest),

    #[api(type = 0x0406)]
    #[display("enclose({0})")]
    Enclose(Disclosure),

    #[api(type = 0x0407)]
    #[display("forget(...)")]
    Forget(Vec<(NodeId, u16)>),
}

#[derive(Clone, StrictEncode, StrictDecode, Debug, Display)]
#[display("consign({contract_id}, ...)")]
pub struct TransferRequest {
    pub contract_id: ContractId,
    pub inputs: BTreeSet<OutPoint>,
    pub transition: Transition,
    pub other_transitions: BTreeMap<ContractId, Transition>,
    pub endpoints: BTreeSet<SealEndpoint>,
    pub psbt: Psbt,
}

#[derive(Clone, StrictEncode, StrictDecode, Debug, Display)]
#[display("accept(...)")]
pub struct AcceptRequest {
    pub consignment: Consignment,
    pub reveal_outpoints: Vec<OutpointReveal>,
}
