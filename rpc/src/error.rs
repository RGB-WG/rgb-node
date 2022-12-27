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

use microservices::{esb, rpc};

use crate::{RpcMsg, ServiceId};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum FailureCode {
    /// Catch-all
    Unknown = 0xFFF,

    ChainMismatch = 0x01,

    /// Encoding
    Encoding = 0x10,

    Esb = 0x11,

    Store = 0x12,

    Stash = 0x13,

    Absent = 0x14,

    Finalize = 0x15,

    ElectrumConnectivity = 0x16,

    UnexpectedRequest = 0x80,

    /// Daemon launcher error
    Launcher = 0x81,
}

impl Display for FailureCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let val = *self as u16;
        Display::fmt(&val, f)
    }
}

impl From<u16> for FailureCode {
    fn from(value: u16) -> Self {
        match value {
            x if x == FailureCode::ChainMismatch as u16 => FailureCode::ChainMismatch,
            x if x == FailureCode::Encoding as u16 => FailureCode::Encoding,
            x if x == FailureCode::Esb as u16 => FailureCode::Esb,
            x if x == FailureCode::Store as u16 => FailureCode::Store,
            x if x == FailureCode::Stash as u16 => FailureCode::Stash,
            x if x == FailureCode::Absent as u16 => FailureCode::Absent,
            x if x == FailureCode::Finalize as u16 => FailureCode::Finalize,
            x if x == FailureCode::ElectrumConnectivity as u16 => FailureCode::ElectrumConnectivity,
            x if x == FailureCode::UnexpectedRequest as u16 => FailureCode::UnexpectedRequest,
            x if x == FailureCode::Launcher as u16 => FailureCode::Launcher,
            _ => FailureCode::Unknown,
        }
    }
}

impl From<FailureCode> for u16 {
    fn from(code: FailureCode) -> Self { code as u16 }
}

impl From<FailureCode> for rpc::FailureCode<FailureCode> {
    fn from(code: FailureCode) -> Self { rpc::FailureCode::Other(code) }
}

impl rpc::FailureCodeExt for FailureCode {}

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    #[display(inner)]
    #[from]
    Esb(esb::Error<ServiceId>),

    /// (RGB#{code:06}) {message}
    LocalFailure { code: FailureCode, message: String },

    /// (EXT#{code:08}) {message}
    RemoteFailure {
        code: rpc::FailureCode<FailureCode>,
        message: String,
    },

    /// unexpected server response
    UnexpectedServerResponse,
}

impl RpcMsg {
    pub fn failure_to_error(self) -> Result<RpcMsg, Error> {
        match self {
            RpcMsg::Failure(rpc::Failure {
                code: rpc::FailureCode::Other(code),
                info,
            }) => Err(Error::LocalFailure {
                code,
                message: info,
            }),
            RpcMsg::Failure(failure) => Err(Error::RemoteFailure {
                code: failure.code,
                message: failure.info,
            }),
            msg => Ok(msg),
        }
    }
}
