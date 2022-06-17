// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::presentation;
use microservices::{esb, rpc};
use rgb_rpc::{FailureCode, RpcMsg};

use crate::bus::{ServiceBus, ServiceId};

#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum LaunchError {
    /// unable to connect LNP node message bus
    NoLnpdConnection,
}

impl microservices::error::Error for LaunchError {}

#[derive(Clone, Debug, Display, Error, From)]
#[display(doc_comments)]
pub(crate) enum DaemonError {
    #[from]
    #[display(inner)]
    Encoding(strict_encoding::Error),

    /// ESB error: {0}
    #[from]
    Esb(esb::Error<ServiceId>),

    /// invalid storm message encoding. Details: {0}
    #[from]
    StormEncoding(presentation::Error),

    /// request `{1}` is not supported on {0} message bus
    RequestNotSupported(ServiceBus, String),

    /// request `{1}` is not supported on {0} message bus for service {2}
    SourceNotSupported(ServiceBus, String, ServiceId),
}

impl microservices::error::Error for DaemonError {}

impl From<DaemonError> for esb::Error<ServiceId> {
    fn from(err: DaemonError) -> Self { esb::Error::ServiceError(err.to_string()) }
}

impl From<DaemonError> for RpcMsg {
    fn from(err: DaemonError) -> Self {
        let code = match err {
            DaemonError::StormEncoding(_) | DaemonError::Encoding(_) => FailureCode::Encoding,
            DaemonError::Esb(_) => FailureCode::Esb,
            DaemonError::RequestNotSupported(_, _) | DaemonError::SourceNotSupported(_, _, _) => {
                FailureCode::UnexpectedRequest
            }
        };
        RpcMsg::Failure(rpc::Failure {
            code: code.into(),
            info: err.to_string(),
        })
    }
}

impl DaemonError {
    pub(crate) fn wrong_esb_msg(bus: ServiceBus, message: &impl ToString) -> DaemonError {
        DaemonError::RequestNotSupported(bus, message.to_string())
    }

    pub(crate) fn wrong_esb_msg_source(
        bus: ServiceBus,
        message: &impl ToString,
        source: ServiceId,
    ) -> DaemonError {
        DaemonError::SourceNotSupported(bus, message.to_string(), source)
    }
}
