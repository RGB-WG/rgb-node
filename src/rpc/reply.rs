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

use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use bitcoin::OutPoint;
use microservices::FileFormat;
use rgb::{AtomicValue, Consignment, ContractId, Disclosure};
use rgb20::Asset;

#[cfg(feature = "node")]
use crate::error::RuntimeError;
#[cfg(any(feature = "node", feature = "client"))]
use crate::error::ServiceError;

#[derive(Clone, Debug, Display, Api)]
#[api(encoding = "strict")]
#[display(inner)]
#[non_exhaustive]
pub enum Reply {
    #[api(type = 0x0003)]
    #[display("success()")]
    Success,

    #[api(type = 0x0001)]
    Failure(crate::rpc::reply::Failure),

    /// There was nothing to do
    #[api(type = 0x0005)]
    #[display("noop()")]
    Nothing,

    #[api(type = 0xFF00)]
    Sync(crate::rpc::reply::SyncFormat),

    #[api(type = 0xFF01)]
    #[display("asset({0})")]
    Asset(Asset),

    #[api(type = 0xFF02)]
    #[display("outpoint_assets(...)")]
    OutpointAssets(BTreeMap<ContractId, Vec<AtomicValue>>),

    #[api(type = 0xFF03)]
    #[display("asset_allocations(...)")]
    AssetAllocations(BTreeMap<OutPoint, Vec<AtomicValue>>),

    #[api(type = 0xFF04)]
    #[display("schema_ids(...)")]
    SchemaIds(Vec<::rgb::SchemaId>),

    #[api(type = 0xFF05)]
    #[display("contract_ids(...)")]
    ContractIds(Vec<::rgb::ContractId>),

    #[api(type = 0xFF07)]
    #[display("genesis({0})")]
    Genesis(::rgb::Genesis),

    #[api(type = 0xFF09)]
    #[display("schema({0})")]
    Schema(::rgb::Schema),

    #[api(type = 0xFF0A)]
    #[display("transitions(...)")]
    Transitions(Vec<::rgb::Transition>),

    #[api(type = 0xFF0C)]
    Transfer(crate::rpc::reply::Transfer),

    #[api(type = 0xFF0B)]
    #[display("validation_status({0})")]
    ValidationStatus(::rgb::validation::Status),
}

impl From<internet2::presentation::Error> for Reply {
    fn from(err: internet2::presentation::Error) -> Self {
        Reply::Failure(Failure::from(err))
    }
}

impl From<internet2::transport::Error> for Reply {
    fn from(err: internet2::transport::Error) -> Self {
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
#[display("sync(using: {0}, ...)")]
pub struct SyncFormat(pub FileFormat, pub Vec<u8>);

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode, Error)]
#[display("transfer(...)")]
pub struct Transfer {
    pub consignment: Consignment,
    pub disclosure: Disclosure,
    pub witness: Psbt,
}

#[derive(Clone, Debug, Display, StrictEncode, StrictDecode, Error)]
#[display("failure({code}, {info})")]
#[non_exhaustive]
pub struct Failure {
    pub code: u16,
    pub info: String,
}

impl From<internet2::presentation::Error> for Failure {
    fn from(err: internet2::presentation::Error) -> Self {
        // TODO #61: Save error code taken from `Error::to_value()` after
        //       implementation of `ToValue` trait and derive macro for enums
        Failure {
            code: 0,
            info: format!("{}", err),
        }
    }
}

impl From<internet2::transport::Error> for Failure {
    fn from(err: internet2::transport::Error) -> Self {
        // TODO #61: Save error code taken from `Error::to_value()` after
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
        // TODO #61: Save error code taken from `Error::to_value()` after
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
        // TODO #61: Save error code taken from `Error::to_value()` after
        //       implementation of `ToValue` trait and derive macro for enums
        Failure {
            code: 3,
            info: format!("{}", err),
        }
    }
}
