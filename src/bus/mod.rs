// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

mod services;
mod ctl;

use microservices::rpc;
use rgb_rpc::RpcMsg;
use storm_ext::ExtMsg as StormMsg;

pub use self::ctl::{
    ConsignReq, CtlMsg, FinalizeTransferReq, FinalizeTransfersReq, OutpointStateReq,
    ProcessDisclosureReq, ProcessReq, ValidityResp,
};
pub use self::services::{DaemonId, ServiceId};
pub(crate) use self::services::{Endpoints, Responder, ServiceBus};

/// Service controller messages
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[display(inner)]
pub(crate) enum BusMsg {
    /// RPC requests
    #[api(type = 4)]
    #[display(inner)]
    #[from]
    Rpc(RpcMsg),

    /// CTL requests
    #[api(type = 6)]
    #[display(inner)]
    #[from]
    Ctl(CtlMsg),

    /// Storm node <-> application extensions messaging
    #[api(type = 5)]
    #[display(inner)]
    #[from]
    Storm(StormMsg),
}

impl rpc::Request for BusMsg {}
