mod asset;
mod manager;
mod outcoins;
pub mod schema;
mod storage;

pub use asset::*;
pub use manager::*;
pub use outcoins::Outcoins;
pub use storage::{DiskStorage, DiskStorageConfig, DiskStorageError, Store};

use super::InteroperableError;

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
