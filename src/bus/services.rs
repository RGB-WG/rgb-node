// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::str::FromStr;

use microservices::esb;
use rgb_rpc::{ClientId, RpcMsg, ServiceName};
use storm_ext::ExtMsg as StormMsg;
use strict_encoding::{strict_deserialize, strict_serialize};

use crate::bus::{BusMsg, CtlMsg};

pub(crate) type Endpoints = esb::EndpointList<ServiceBus>;

pub type DaemonId = u64;

/// Identifiers of daemons participating in LNP Node
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, From, StrictEncode, StrictDecode)]
pub enum ServiceId {
    #[display("rgbd")]
    Rgb,

    #[display("client<{0}>")]
    Client(ClientId),

    #[display("bpd")]
    Bp,

    #[display("contained<{0}>")]
    Container(DaemonId),

    #[display("stormd")]
    Storm,

    #[display("other<{0}>")]
    Other(ServiceName),
}

impl esb::ServiceAddress for ServiceId {}

impl From<ServiceId> for Vec<u8> {
    fn from(daemon_id: ServiceId) -> Self {
        strict_serialize(&daemon_id).expect("Memory-based encoding does not fail")
    }
}

impl From<Vec<u8>> for ServiceId {
    fn from(vec: Vec<u8>) -> Self {
        strict_deserialize(&vec).unwrap_or_else(|_| {
            ServiceId::Other(
                ServiceName::from_str(&String::from_utf8_lossy(&vec))
                    .expect("ClientName conversion never fails"),
            )
        })
    }
}

/// Service buses used for inter-daemon communication
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub(crate) enum ServiceBus {
    /// RPC interface, from client to node
    #[display("RPC")]
    Rpc,

    #[display("CTL")]
    Ctl,

    /// Storm P2P message bus
    #[display("STORM")]
    Storm,

    /// BP node message bus
    #[display("BP")]
    Bp,
}

impl esb::BusId for ServiceBus {
    type Address = ServiceId;
}

pub(crate) trait Responder
where
    Self: esb::Handler<ServiceBus>,
    esb::Error<ServiceId>: From<Self::Error>,
{
    #[inline]
    fn send_rpc(
        &self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        message: impl Into<RpcMsg>,
    ) -> Result<(), esb::Error<ServiceId>> {
        endpoints.send_to(
            ServiceBus::Rpc,
            self.identity(),
            ServiceId::Client(client_id),
            BusMsg::Rpc(message.into()),
        )
    }

    #[inline]
    fn send_ctl(
        &self,
        endpoints: &mut Endpoints,
        service_id: ServiceId,
        message: impl Into<CtlMsg>,
    ) -> Result<(), esb::Error<ServiceId>> {
        endpoints.send_to(ServiceBus::Ctl, self.identity(), service_id, BusMsg::Ctl(message.into()))
    }

    #[inline]
    fn send_storm(
        &self,
        endpoints: &mut Endpoints,
        message: impl Into<StormMsg>,
    ) -> Result<(), esb::Error<ServiceId>> {
        endpoints.send_to(
            ServiceBus::Storm,
            self.identity(),
            ServiceId::Storm,
            BusMsg::Storm(message.into()),
        )
    }
}
