// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::BTreeSet;

use internet2::presentation;
use microservices::rpc;
use rgb::{Contract, ContractId, ContractState, StateTransfer};

use crate::FailureCode;

/// We need this wrapper type to be compatible with RGB Node having multiple message buses
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub(crate) enum BusMsg {
    #[api(type = 4)]
    #[display(inner)]
    #[from]
    Rpc(RpcMsg),
}

impl rpc::Request for BusMsg {}

#[derive(Clone, Eq, PartialEq, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[display(inner)]
pub enum RpcMsg {
    // Contract operations
    // -------------------
    #[display("add_contract(...)")]
    AddContract(Contract),

    #[display("list_contracts")]
    ListContracts,

    #[display("list_contract_ids")]
    ListContractIds,

    #[display("get_contract({0})")]
    GetContact(ContractId),

    #[display("get_contract_state({0})")]
    GetContractState(ContractId),

    // Stash operations
    // ----------------
    BlindUtxo,

    ComposeTransfer,

    AcceptTransfer,

    // Responses to CLI
    // ----------------
    #[display("contract_ids(...)")]
    ContractIds(BTreeSet<ContractId>),

    #[display("contract(...)")]
    Contract(Contract),

    #[display("contract_state(...)")]
    ContractState(ContractState),

    #[display("state_transfer(...)")]
    StateTransfer(StateTransfer),

    #[display("success({0})")]
    Success,

    #[display("failure({0:#})")]
    #[from]
    Failure(rpc::Failure<FailureCode>),
}

impl From<presentation::Error> for RpcMsg {
    fn from(err: presentation::Error) -> Self {
        RpcMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Presentation,
            info: format!("{}", err),
        })
    }
}
