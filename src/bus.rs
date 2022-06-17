// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use amplify::hex::{self, ToHex};
use microservices::{esb, rpc};
use rgb_rpc::RpcMsg;
use storm_app::AppMsg as StormMsg;
use strict_encoding::{strict_deserialize, strict_serialize};

pub(crate) type Endpoints = esb::EndpointList<ServiceBus>;

pub type ClientId = u64;
pub type DaemonId = u64;

#[derive(Wrapper, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Default)]
#[derive(StrictEncode, StrictDecode)]
pub struct ServiceName([u8; 32]);

impl Display for ServiceName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}..{}", self.0[..4].to_hex(), self.0[(self.0.len() - 4)..].to_hex())
        } else {
            f.write_str(&String::from_utf8_lossy(&self.0))
        }
    }
}

impl FromStr for ServiceName {
    type Err = hex::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 32 {
            let mut me = Self::default();
            me.0.copy_from_slice(&s.as_bytes()[0..32]);
            Ok(me)
        } else {
            let mut me = Self::default();
            me.0[0..s.len()].copy_from_slice(s.as_bytes());
            Ok(me)
        }
    }
}

/// Identifiers of daemons participating in LNP Node
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, From, StrictEncode, StrictDecode)]
pub enum ServiceId {
    #[display("rgbd")]
    Rgb,

    #[display("client<{0}>")]
    Client(ClientId),

    #[display("bpd")]
    Bp,

    #[display("stormd")]
    Container(DaemonId),

    #[display("stormd")]
    Storm,

    #[display("other<{0}>")]
    Other(ServiceName),
}

impl esb::ServiceAddress for ServiceId {}

/// Service buses used for inter-daemon communication
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub(crate) enum ServiceBus {
    /// RPC interface, from client to node
    #[display("RPC")]
    Rpc,

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

    /// Storm node <-> application extensions messaging
    #[api(type = 5)]
    #[display(inner)]
    #[from]
    Storm(StormMsg),
}

impl rpc::Request for BusMsg {}

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
