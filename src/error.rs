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

// TODO #152: Consider moving parts of this file to common daemon modules
//       (Internet2)

use std::collections::HashMap;
use std::io;

#[derive(Debug, Display, Error, From)]
#[display(Debug)]
#[non_exhaustive]
pub enum BootstrapError {
    TorNotYetSupported,

    #[from]
    IoError(io::Error),

    #[from]
    ArgParseError(String),

    #[from]
    MessageBusError(internet2::transport::Error),

    #[cfg(feature = "electrum-client")]
    #[from]
    ElectrumError(electrum_client::Error),

    StorageError,

    #[cfg(feature = "fungibles")]
    #[from(crate::fungibled::FileCacheError)]
    #[cfg_attr(feature = "sql", from(crate::fungibled::SqlCacheError))]
    CacheError,

    Other,
}

impl From<&str> for BootstrapError {
    fn from(err: &str) -> Self {
        BootstrapError::ArgParseError(err.to_string())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From)]
#[display(Debug)]
#[non_exhaustive]
pub enum RuntimeError {
    #[from(std::io::Error)]
    Io(amplify::IoError),

    #[from]
    Lnp(internet2::transport::Error),

    #[from(internet2::presentation::Error)]
    BrokenTransport,

    Internal(String),
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display(Debug)]
#[non_exhaustive]
pub enum RoutedError {
    Global(RuntimeError),
    RequestSpecific(ServiceError),
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From)]
#[display(Debug)]
#[non_exhaustive]
pub enum ServiceErrorDomain {
    #[from(::std::io::Error)]
    Io(amplify::IoError),

    Stash,

    Storage(String),

    Index(String),

    #[cfg(feature = "fungibles")]
    #[from(crate::fungibled::FileCacheError)]
    #[cfg_attr(feature = "sql", from(crate::fungibled::SqlCacheError))]
    Cache,

    Multithreading,

    P2pwire,

    #[from]
    LnpRpc(internet2::presentation::Error),

    #[from]
    LnpTransport(internet2::transport::Error),

    Api(ApiErrorType),

    Monitoring,

    Bifrost,

    BpNode,

    LnpNode,

    Bitcoin,

    Lightning,

    Electrum,

    Schema(String),

    Anchor(String),

    #[from]
    #[cfg_attr(
        feature = "fungibles",
        from(rgb20::transitions::Error),
        from(rgb20::asset::Error)
    )]
    Internal(String),
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
#[non_exhaustive]
pub enum ServiceErrorSource {
    Broker,
    Stash,
    Contract(String),
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
#[non_exhaustive]
pub enum ServiceSocketType {
    Request,
    Reply,
    Publish,
    Subscribe,
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display(Debug)]
#[non_exhaustive]
pub enum ApiErrorType {
    MalformedRequest { request: String },
    UnknownCommand { command: String },
    UnimplementedCommand,
    MissedArgument { request: String, argument: String },
    UnknownArgument { request: String, argument: String },
    MalformedArgument { request: String, argument: String },
    UnexpectedReply,
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display(Debug)]
#[non_exhaustive]
pub struct ServiceError {
    pub domain: ServiceErrorDomain,
    pub service: ServiceErrorSource,
}

impl ServiceError {
    pub fn contract(domain: ServiceErrorDomain, contract_name: &str) -> Self {
        Self {
            domain,
            service: ServiceErrorSource::Contract(contract_name.to_string()),
        }
    }

    pub fn from_rpc(
        service: ServiceErrorSource,
        err: internet2::presentation::Error,
    ) -> Self {
        Self {
            domain: ServiceErrorDomain::from(err),
            service,
        }
    }
}

#[derive(Debug, Display, Error)]
#[display(Debug)]
#[non_exhaustive]
pub struct ServiceErrorRepresentation {
    pub domain: String,
    pub service: String,
    pub name: String,
    pub description: String,
    pub info: HashMap<String, String>,
}
