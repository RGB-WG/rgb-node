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
use microservices::esb;
use strict_encoding::{strict_deserialize, strict_serialize};

pub type ClientId = u64;

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
