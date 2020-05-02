// For now this is a mod, but later will be a library

pub mod fungible;
#[macro_use]
mod macros;
pub mod file;
mod magic_numbers;
mod storage;

pub use lnpbp::rgb;
pub use magic_numbers::MagicNumber;
pub use rgb::prelude::*;
pub use storage::{DiskStorage, DiskStorageConfig, DiskStorageError, Error, Store};
