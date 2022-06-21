// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use microservices::{esb, rpc};

use crate::ServiceId;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum FailureCode {
    /// Catch-all
    Unknown = 0xFFF,

    /// Encoding
    Encoding = 0x02,

    Esb = 0x03,

    Store = 0x04,

    /// Daemon launcher error
    Launcher = 0x05,

    UnexpectedRequest = 0x06,
}

impl From<u16> for FailureCode {
    fn from(value: u16) -> Self {
        match value {
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

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
#[non_exhaustive]
pub enum Error {
    /// ESB error: {0}
    #[from]
    Esb(esb::Error<ServiceId>),

    /// RPC error: {0}
    #[from]
    Server(rpc::ServerError<FailureCode>),

    /// other error type with string explanation
    #[display(inner)]
    #[from(internet2::addr::NoOnionSupportError)]
    Other(String),
}

impl From<Error> for esb::Error<ServiceId> {
    fn from(err: Error) -> Self {
        match err {
            Error::Esb(err) => err,
            err => esb::Error::ServiceError(err.to_string()),
        }
    }
}
