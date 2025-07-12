// RGB Node: sovereign smart contracts backend
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
// Written in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2020-2024 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2025 RGB Consortium, Switzerland. All rights reserved.
// Copyright (C) 2020-2025 Dr Maxim Orlovsky. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use std::fmt;
use std::fmt::Display;
use std::net::TcpStream;

use bpstd::Network;
use cyphernet::addr::{InetHost, NetAddr};
use indexmap::IndexMap;

pub type Session = TcpStream;
pub type RemoteAddr = NetAddr<InetHost>;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display(doc_comments)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum FailureCode {
    /// Network mismatch
    NetworkMismatch = 1,

    /// Not found
    NotFound = 2,

    /// The result is too large to be encoded and sent in the response
    TooLarge = 3,

    /// HELLO message is required from the client before any other message
    NoHello = 4,

    /// Internal server error
    InternalError = 0xFF,
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Failure {
    pub code: u8,
    pub message: String,
    pub details: IndexMap<String, String>,
}

impl Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failure #{}: message={}", self.code, self.message)?;
        if self.details.is_empty() {
            return Ok(());
        }
        f.write_str("Details:")?;
        for (key, val) in &self.details {
            write!(f, "  {}: {}", key, val)?;
        }
        Ok(())
    }
}

impl Failure {
    pub fn new(code: FailureCode) -> Self {
        Self {
            code: code as u8,
            message: code.to_string(),
            details: Default::default(),
        }
    }

    pub fn with(code: FailureCode, message: &str) -> Self {
        Self {
            code: code as u8,
            message: message.to_string(),
            details: Default::default(),
        }
    }

    pub fn add_detail(mut self, key: &'static str, value: impl ToString) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }

    pub fn network_mismatch() -> Self { Self::new(FailureCode::NetworkMismatch) }

    pub fn not_found(id: impl ToString) -> Self {
        Self::new(FailureCode::NotFound).add_detail("id", id)
    }

    pub fn too_large(id: impl ToString) -> Self {
        Self::new(FailureCode::TooLarge).add_detail("id", id)
    }

    pub fn no_hello() -> Self { Self::new(FailureCode::NoHello) }

    pub fn internal_error(msg: &str) -> Self { Self::with(FailureCode::InternalError, msg) }
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    /// Information about the agent used by the client
    pub agent: Option<AgentInfo>,
    /// The remote client address
    pub remote: RemoteAddr,
    /// Millisecond-based timestamp
    pub connected: u64,
    /// Millisecond-based timestamp
    pub last_seen: u64,
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Status {
    pub clients: Vec<ClientInfo>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display("{agent} v{version} on {network} (features {features:08x})")]
#[derive(Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent: String,
    pub version: Version,
    pub network: Network,
    pub features: u64,
}

impl AgentInfo {
    pub fn new(network: Network, agent: &str, major: u16, minor: u16, patch: u16) -> Self {
        Self {
            agent: agent.to_string(),
            version: Version::new(major, minor, patch),
            network,
            features: 0,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, Default)]
#[display("{major}.{minor}.{patch}")]
#[derive(Serialize, Deserialize)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self { Self { major, minor, patch } }
}
