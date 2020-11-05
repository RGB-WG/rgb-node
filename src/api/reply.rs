// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::BTreeMap;

use lnpbp::bitcoin::OutPoint;
use lnpbp::bp::Psbt;
use lnpbp::lnp;
use lnpbp::rgb::{AtomicValue, Consignment, ContractId};

use crate::DataFormat;

#[cfg(feature = "node")]
use crate::error::RuntimeError;
#[cfg(any(feature = "node", feature = "client"))]
use crate::error::ServiceError;

#[derive(Clone, Debug, Display, LnpApi)]
#[lnp_api(encoding = "strict")]
#[display(Debug)]
#[non_exhaustive]
pub enum Reply {
    #[lnp_api(type = 0x0003)]
    Success,

    #[lnp_api(type = 0x0001)]
    Failure(crate::api::reply::Failure),

    /// There was nothing to do
    #[lnp_api(type = 0x0005)]
    Nothing,

    #[lnp_api(type = 0xFF01)]
    Sync(crate::api::reply::SyncFormat),

    #[lnp_api(type = 0xFF02)]
    Assets(BTreeMap<ContractId, Vec<AtomicValue>>),

    #[lnp_api(type = 0xFF03)]
    Allocations(BTreeMap<OutPoint, Vec<AtomicValue>>),

    #[lnp_api(type = 0xFF04)]
    SchemaIds(Vec<::lnpbp::rgb::SchemaId>),

    #[lnp_api(type = 0xFF05)]
    ContractIds(Vec<::lnpbp::rgb::ContractId>),

    #[lnp_api(type = 0xFF07)]
    Genesis(::lnpbp::rgb::Genesis),

    #[lnp_api(type = 0xFF09)]
    Schema(::lnpbp::rgb::Schema),

    #[lnp_api(type = 0xFF0A)]
    Transitions(Vec<::lnpbp::rgb::Transition>),

    #[lnp_api(type = 0xFF0C)]
    Transfer(crate::api::reply::Transfer),
    /* #[lnp_api(type = 0xFF0B)]
    ValidationStatus(::lnpbp::rgb::validation::Status), */
}

impl From<lnp::presentation::Error> for Reply {
    fn from(err: lnp::presentation::Error) -> Self {
        Reply::Failure(Failure::from(err))
    }
}

impl From<lnp::transport::Error> for Reply {
    fn from(err: lnp::transport::Error) -> Self {
        Reply::Failure(Failure::from(err))
    }
}

#[cfg(feature = "node")]
impl From<RuntimeError> for Reply {
    fn from(err: RuntimeError) -> Self {
        Reply::Failure(Failure::from(err))
    }
}

#[cfg(any(feature = "node", feature = "client"))]
impl From<ServiceError> for Reply {
    fn from(err: ServiceError) -> Self {
        Reply::Failure(Failure::from(err))
    }
}

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode, Error)]
#[display(Debug)]
pub struct SyncFormat(pub DataFormat, pub Vec<u8>);

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode, Error)]
#[display(Debug)]
pub struct Transfer {
    pub consignment: Consignment,
    pub psbt: Psbt,
}

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode, Error)]
#[display(Debug)]
#[non_exhaustive]
pub struct Failure {
    pub code: u16,
    pub info: String,
}

impl From<lnp::presentation::Error> for Failure {
    fn from(err: lnp::presentation::Error) -> Self {
        // TODO: Save error code taken from `Error::to_value()` after
        //       implementation of `ToValue` trait and derive macro for enums
        Failure {
            code: 0,
            info: format!("{}", err),
        }
    }
}

impl From<lnp::transport::Error> for Failure {
    fn from(err: lnp::transport::Error) -> Self {
        // TODO: Save error code taken from `Error::to_value()` after
        //       implementation of `ToValue` trait and derive macro for enums
        Failure {
            code: 1,
            info: format!("{}", err),
        }
    }
}

#[cfg(feature = "node")]
impl From<RuntimeError> for Failure {
    fn from(err: RuntimeError) -> Self {
        // TODO: Save error code taken from `Error::to_value()` after
        //       implementation of `ToValue` trait and derive macro for enums
        Failure {
            code: 2,
            info: format!("{}", err),
        }
    }
}

#[cfg(any(feature = "node", feature = "client"))]
impl From<ServiceError> for Failure {
    fn from(err: ServiceError) -> Self {
        // TODO: Save error code taken from `Error::to_value()` after
        //       implementation of `ToValue` trait and derive macro for enums
        Failure {
            code: 3,
            info: format!("{}", err),
        }
    }
}
