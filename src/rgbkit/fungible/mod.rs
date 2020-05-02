mod asset;
mod manager;
pub mod schema;
mod storage;

pub use asset::*;
pub use manager::*;
pub use storage::{DiskStorage, Store};

use crate::rgbkit::storage::Error as RgbStoreError;
use storage::Error as AssetStoreError;

#[derive(Debug, Display, Error, From)]
#[display_from(Display)]
pub enum SchemaError {
    #[derive_from(core::option::NoneError)]
    NotAllFieldsPresent,
}

#[derive(Debug, Display, Error, From)]
#[display_from(Display)]
pub enum Error<E1, E2>
where
    E1: RgbStoreError,
    E2: AssetStoreError,
{
    RgbStorage(E1),

    AssetStorage(E2),

    #[derive_from]
    Secp(lnpbp::secp256k1zkp::Error),

    #[derive_from]
    SchemaError(SchemaError),
}

impl<E1, E2> From<E1> for Error<E1, E2>
where
    E1: RgbStoreError,
    E2: AssetStoreError,
{
    fn from(err: E1) -> Self {
        Self::RgbStorage(err)
    }
}
/*
impl<E1, E2> From<E2> for Error<E1, E2>
where
    E1: RgbStoreError,
    E2: AssetStoreError,
{
    fn from(err: E2) -> Self {
        Self::AssetStorage(err)
    }
}
*/
