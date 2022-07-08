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
use microservices::esb::{ClientId, ServiceName};
use strict_encoding::{strict_deserialize, strict_serialize};

/// Identifiers of daemons participating in RGB Node
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, From, StrictEncode, StrictDecode)]
pub enum ServiceId {
    #[display("rgbd")]
    Rgb,

    #[display("client<{0}>")]
    Client(ClientId),

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

impl ServiceId {
    pub fn router() -> Self { ServiceId::Rgb }
}
