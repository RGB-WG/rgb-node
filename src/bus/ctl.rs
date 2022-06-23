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

use internet2::addr::NodeAddr;
use psbt::Psbt;
use rgb::schema::TransitionType;
use rgb::{
    validation, ConsignmentId, ConsignmentType, ContractConsignment, ContractId, InmemConsignment,
    StateTransfer, TransferConsignment, Transition,
};
use rgb_rpc::{ClientId, OutpointFilter};

/// RPC API requests over CTL message bus between RGB Node daemons.
#[derive(Clone, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[non_exhaustive]
pub enum CtlMsg {
    #[display("hello()")]
    Hello,

    #[display("process_contract({0})")]
    ProcessContract(ProcessReq<ContractConsignment>),

    #[display("process_transfer({0})")]
    ProcessTransfer(ProcessReq<TransferConsignment>),

    #[display("consign_contract({0})")]
    ConsignContract(ConsignReq<ContractConsignment>),

    #[display("consign_transition({0})")]
    ConsignTranfer(ConsignReq<TransferConsignment>),

    #[display(inner)]
    ProcessPsbt(ProcessPsbtReq),

    #[display(inner)]
    FinalizeTransfer(FinalizeTransferReq),

    #[display(inner)]
    #[from]
    Validity(ValidityResp),

    #[display("processing_complete()")]
    ProcessingComplete,

    #[display("processing_failed()")]
    ProcessingFailed,
}

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode)]
#[display("{client_id}, force = {force}, ...")]
pub struct ProcessReq<T: ConsignmentType> {
    pub client_id: ClientId,
    pub consignment: InmemConsignment<T>,
    pub force: bool,
}

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode)]
#[display("{client_id}, {contract_id}, ...")]
pub struct ConsignReq<T: ConsignmentType> {
    pub client_id: ClientId,
    pub contract_id: ContractId,
    pub include: BTreeSet<TransitionType>,
    pub outpoints: OutpointFilter,
    #[strict_encoding(skip)]
    pub _phantom: T,
}

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode)]
#[display("validity({client_id}, {consignment_id}, ...)")]
pub struct ValidityResp {
    pub client_id: ClientId,
    pub consignment_id: ConsignmentId,
    pub status: validation::Status,
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("process_psbt(...)")]
pub struct ProcessPsbtReq {
    pub client_id: ClientId,
    pub transition: Transition,
    pub psbt: Psbt,
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("finalize_transfer(...)")]
pub struct FinalizeTransferReq {
    pub client_id: ClientId,
    pub consignment: StateTransfer,
    pub psbt: Psbt,
    pub beneficiary: Option<NodeAddr>,
}
