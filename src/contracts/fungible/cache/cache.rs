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

use super::FileCacheError;
use crate::fungible::Asset;
use crate::util::file::FileMode;
use lnpbp::rgb::prelude::*;

pub trait Cache {
    fn assets(&self) -> Result<Vec<&Asset>, CacheError>;
    fn asset(&self, id: ContractId) -> Result<&Asset, CacheError>;
    fn has_asset(&self, id: ContractId) -> Result<bool, CacheError>;
    fn add_asset(&mut self, asset: Asset) -> Result<bool, CacheError>;
    fn remove_asset(&mut self, id: ContractId) -> Result<bool, CacheError>;
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display_from(Debug)]
pub enum CacheError {
    Io(String),
    NotFound {
        id: String,
    },
    DataAccessError {
        id: String,
        mode: FileMode,
        details: Option<String>,
    },
    DataIntegrityError(String),
}

impl From<FileCacheError> for CacheError {
    fn from(err: FileCacheError) -> Self {
        match err {
            FileCacheError::Io(e) => Self::Io(format!("{:?}", e)),
            FileCacheError::HashName => {
                Self::DataIntegrityError("File for a given hash id is not found".to_string())
            }
            FileCacheError::Encoding(e) => Self::DataIntegrityError(format!("{:?}", e)),
            FileCacheError::BrokenHexFilenames => {
                Self::DataIntegrityError("Broken filename structure in storage".to_string())
            }
            FileCacheError::SerdeJson(e) => Self::DataIntegrityError(format!("{:?}", e)),
            FileCacheError::NotFound => {
                Self::DataIntegrityError("Data file is not found".to_string())
            }
        }
    }
}
