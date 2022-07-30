// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use commit_verify::lnpbp4;
use internet2::presentation;
use microservices::rpc::ServerError;
use microservices::{esb, rpc, LauncherError};
use rgb_rpc::{FailureCode, RpcMsg};
use storm::ContainerId;

use crate::bucketd::{FinalizeError, StashError};
use crate::bus::{ServiceBus, ServiceId};
use crate::rgbd::Daemon;

#[derive(Clone, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum LaunchError {
    /// error setting up ESB controller; can't connect one of message buses
    BusSetupFailure,

    /// can't connect to store service. Details: {0}
    #[from]
    StoreConnection(ServerError<store_rpc::FailureCode>),

    /// can't connect to electrum server
    ElectrumConnectivity,
}

impl microservices::error::Error for LaunchError {}

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub(crate) enum DaemonError {
    /// data encoding error. Details: {0}
    #[from]
    #[display(inner)]
    Encoding(strict_encoding::Error),

    /// ESB error: {0}
    #[from]
    Esb(esb::Error<ServiceId>),

    /// invalid storm message encoding. Details: {0}
    #[from]
    StormEncoding(presentation::Error),

    /// launching bucket daemon. Details: {0}
    #[from]
    BucketLauncher(LauncherError<Daemon>),

    /// storage error. Details: {0}
    #[from]
    Store(ServerError<store_rpc::FailureCode>),

    #[display(inner)]
    #[from]
    #[from(lnpbp4::LeafNotKnown)]
    #[from(rgb::bundle::RevealError)]
    Stash(StashError),

    #[display(inner)]
    #[from]
    #[from(rgb::psbt::KeyError)]
    #[from(bp::dbc::anchor::Error)]
    Finalize(FinalizeError),

    /// the container which was requested to be processed is absent in sthe store
    NoContainer(ContainerId),

    /// request `{1}` is not supported on {0} message bus
    RequestNotSupported(ServiceBus, String),
    // /// request `{1}` is not supported on {0} message bus for service {2}
    // SourceNotSupported(ServiceBus, String, ServiceId),
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
            DaemonError::RequestNotSupported(_, _) /* | DaemonError::SourceNotSupported(_, _, _) */ => {
                FailureCode::UnexpectedRequest
            }
            DaemonError::Store(_) => FailureCode::Store,
            DaemonError::BucketLauncher(_) => FailureCode::Launcher,
            DaemonError::Stash(_) => FailureCode::Stash,
            DaemonError::Finalize(_) => FailureCode::Finalize,
            DaemonError::NoContainer(_) => FailureCode::Store,
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

    /*
    pub(crate) fn wrong_esb_msg_source(
        bus: ServiceBus,
        message: &impl ToString,
        source: ServiceId,
    ) -> DaemonError {
        DaemonError::SourceNotSupported(bus, message.to_string(), source)
    }
     */
}
