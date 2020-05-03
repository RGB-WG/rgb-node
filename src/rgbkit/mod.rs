// For now this is a mod, but later will be a library

pub mod fungible;
#[macro_use]
mod macros;
mod error;
pub mod file;
mod magic_numbers;
mod seal_spec;
mod storage;

pub use error::{InteroperableError, ParseError};
pub use magic_numbers::MagicNumber;
pub use seal_spec::SealSpec;
pub use storage::{DiskStorage, DiskStorageConfig, DiskStorageError, Error as StoreError, Store};

pub const RGB_BECH32_HRP_GENESIS: &'static str = "rgb:genesis";
