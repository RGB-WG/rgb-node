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

use lnpbp::bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use lnpbp::bitcoin::OutPoint;
use lnpbp::rgb::{ContractId, Transition};

#[derive(Clone, Debug, Display, LnpApi)]
#[lnp_api(encoding = "strict")]
#[display_from(Debug)]
#[non_exhaustive]
pub enum Request {
    #[lnp_api(type = 0x0101)]
    AddSchema(::lnpbp::rgb::Schema),

    #[lnp_api(type = 0x0103)]
    ListSchemata(),

    #[lnp_api(type = 0x0105)]
    ReadSchemata(Vec<::lnpbp::rgb::SchemaId>),

    #[lnp_api(type = 0x0201)]
    AddGenesis(::lnpbp::rgb::Genesis),

    #[lnp_api(type = 0x0203)]
    ListGeneses(),

    #[lnp_api(type = 0x0205)]
    ReadGenesis(::lnpbp::rgb::ContractId),

    #[lnp_api(type = 0x0301)]
    ReadTransitions(Vec<::lnpbp::rgb::TransitionId>),

    #[lnp_api(type = 0x0401)]
    Consign(crate::api::stash::ConsignRequest),
    /*
    #[lnp_api(type = 0x0403)]
    MergeConsignment(::lnpbp::rgb::Consignment),

    #[lnp_api(type = 0x0405)]
    VerifyConsignment(::lnpbp::rgb::Consignment),

    #[lnp_api(type = 0x0407)]
    ForgetConsignment(::lnpbp::rgb::Consignment),
     */
}

#[derive(Clone, StrictEncode, StrictDecode, Debug, Display)]
#[display_from(Debug)]
pub struct ConsignRequest {
    pub contract_id: ContractId,
    pub inputs: Vec<OutPoint>,
    pub transition: Transition,
    pub psbt: Psbt,
}
