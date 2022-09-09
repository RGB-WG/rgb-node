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

use bitcoin::{OutPoint, Txid};
use internet2::addr::NodeAddr;
use internet2::presentation;
use lnpbp::chain::Chain;
use microservices::rpc;
use microservices::util::OptionDetails;
use psbt::Psbt;
use rgb::schema::TransitionType;
use rgb::{
    seal, validation, ConsignmentType, Contract, ContractConsignment, ContractId, ContractState,
    ContractStateMap, InmemConsignment, SealEndpoint, StateTransfer, TransferConsignment,
};

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

#[derive(Clone, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[display(inner)]
pub enum RpcMsg {
    #[from]
    Hello(HelloReq),

    // Contract operations
    // -------------------
    #[display("list_contracts")]
    ListContracts,

    #[display("get_contract_state({0})")]
    GetContractState(ContractId),

    #[display("get_outpoint_state(...)")]
    GetOutpointState(BTreeSet<OutPoint>),

    #[display("consign_contract({0})")]
    ConsignContract(ComposeReq),

    #[display("consign_transfer({0})")]
    ConsignTransfer(ComposeReq),

    #[display(inner)]
    ConsumeContract(AcceptReq<ContractConsignment>),

    #[display("accept_transfer(...)")]
    ConsumeTransfer(AcceptReq<TransferConsignment>),

    #[display(inner)]
    Transfer(TransferReq),

    #[display("memorize_seal({0})")]
    MemorizeSeal(seal::Revealed),

    // Responses to CLI
    // ----------------
    #[display("contract_ids(...)")]
    ContractIds(BTreeSet<ContractId>),

    #[display("contract(...)")]
    Contract(Contract),

    #[display("contract_state(...)")]
    ContractState(ContractState),

    #[display("outpoint_state(...)")]
    OutpointState(ContractStateMap),

    #[display("state_transfer(...)")]
    StateTransfer(StateTransfer),

    #[display("state_transfer_finalize(...)")]
    StateTransferFinalize(TransferFinalize),

    #[display("progress(\"{0}\")")]
    #[from]
    Progress(String),

    #[display("success{0}")]
    Success(OptionDetails),

    #[display("failure({0:#})")]
    #[from]
    Failure(rpc::Failure<FailureCode>),

    #[display("unresolved_txids(...)")]
    UnresolvedTxids(Vec<Txid>),

    #[display("invalid(...)")]
    Invalid(validation::Status),
}

impl From<presentation::Error> for RpcMsg {
    fn from(err: presentation::Error) -> Self {
        RpcMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Presentation,
            info: format!("{}", err),
        })
    }
}

impl RpcMsg {
    pub fn success() -> Self { RpcMsg::Success(None.into()) }
    pub fn failure(code: FailureCode, message: impl ToString) -> Self {
        RpcMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Other(code),
            info: message.to_string(),
        })
    }
}

#[derive(Clone, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub enum ContractValidity {
    Valid,
    Invalid(validation::Status),
    UnknownTxids(Vec<Txid>),
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub enum OutpointFilter {
    All,
    Only(BTreeSet<OutPoint>),
}

impl OutpointFilter {
    pub fn includes(&self, outpoint: OutPoint) -> bool {
        match self {
            OutpointFilter::All => true,
            OutpointFilter::Only(set) => set.contains(&outpoint),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("accept(force: {force}, ...)")]
pub struct AcceptReq<T: ConsignmentType> {
    pub consignment: InmemConsignment<T>,
    pub force: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("hello({network}, {user_agent})")]
pub struct HelloReq {
    pub user_agent: String,
    pub network: Chain,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("{contract_id}, ...")]
pub struct ComposeReq {
    pub contract_id: ContractId,
    pub include: BTreeSet<TransitionType>,
    pub outpoints: OutpointFilter,
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("transfer(...)")]
pub struct TransferReq {
    pub consignment: StateTransfer,
    pub endseals: Vec<SealEndpoint>,
    pub psbt: Psbt,
    pub beneficiary: Option<NodeAddr>,
}

impl From<&str> for RpcMsg {
    fn from(s: &str) -> Self { RpcMsg::Progress(s.to_owned()) }
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("transfer_complete(...)")]
pub struct TransferFinalize {
    pub consignment: StateTransfer,
    pub psbt: Psbt,
}
