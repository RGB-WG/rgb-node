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

mod asset;
mod config;
mod outcoins;
mod processor;
mod request;
mod runtime;
pub mod schema;
mod storage;

pub use asset::*;
pub use config::Config;
pub use outcoins::Outcoins;
pub use processor::Processor;
pub use request::Request;
pub use runtime::Runtime;
pub use storage::{DiskStorage, DiskStorageConfig, DiskStorageError, Store};

#[derive(Debug, Display, Error, From)]
#[display_from(Display)]
pub enum SchemaError {
    #[derive_from(core::option::NoneError)]
    NotAllFieldsPresent,
}

#[derive(Debug, Display, Error, From)]
#[display_from(Display)]
pub enum Error {
    InteroperableError(String),

    #[derive_from]
    Secp(lnpbp::secp256k1zkp::Error),

    #[derive_from]
    SchemaError(SchemaError),
}

impl From<InteroperableError> for Error {
    fn from(err: InteroperableError) -> Self {
        Self::InteroperableError(err.0)
    }
}
