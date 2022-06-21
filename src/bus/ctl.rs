// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use microservices::rpc;
use rgb::{
    validation, ConsignmentId, ConsignmentType, Contract, ContractConsignment, InmemConsignment,
    StateTransfer, TransferConsignment,
};
use rgb_rpc::{ClientId, FailureCode};

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

    #[display(inner)]
    #[from]
    Validity(ValidityResp),

    #[display("processing_failed()")]
    ProcessingFailed,
}

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode)]
#[display("{client_id}, ...")]
pub struct ProcessReq<T: ConsignmentType> {
    pub client_id: ClientId,
    pub consignment: InmemConsignment<T>,
}

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode)]
#[display("validity({client_id}, {consignment_id}, ...)")]
pub struct ValidityResp {
    pub client_id: ClientId,
    pub consignment_id: ConsignmentId,
    pub status: validation::Status,
}
