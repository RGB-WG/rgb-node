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

use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io, io::Read, io::Write};

use lnpbp::bitcoin;
use lnpbp::data_format::DataFormat;
use lnpbp::rgb::prelude::*;

use super::Cache;
use crate::error::{BootstrapError, ServiceErrorDomain};
use crate::fungible::cache::CacheError;
use crate::fungible::Asset;
use crate::util::file::*;

#[derive(Debug, Display, Error, From)]
#[display_from(Debug)]
pub enum FileCacheError {
    #[derive_from]
    Io(io::Error),

    #[derive_from(bitcoin::hashes::Error)]
    HashName,

    #[derive_from(bitcoin::hashes::hex::Error)]
    BrokenHexFilenames,

    #[derive_from]
    Encoding(lnpbp::strict_encoding::Error),

    #[derive_from]
    SerdeJson(serde_json::Error),

    #[derive_from]
    SerdeYaml(serde_yaml::Error),

    #[derive_from(toml::de::Error, toml::ser::Error)]
    SerdeToml,

    #[derive_from(std::option::NoneError)]
    NotFound,
}

impl From<FileCacheError> for ServiceErrorDomain {
    fn from(_: FileCacheError) -> Self {
        ServiceErrorDomain::Cache
    }
}

impl From<FileCacheError> for BootstrapError {
    fn from(_: FileCacheError) -> Self {
        BootstrapError::CacheError
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display_from(Debug)]
pub struct FileCacheConfig {
    pub data_dir: PathBuf,
    pub data_format: DataFormat,
}

impl FileCacheConfig {
    #[inline]
    pub fn assets_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    #[inline]
    pub fn assets_filename(&self) -> PathBuf {
        self.assets_dir()
            .join("assets")
            .with_extension(self.data_format.extension())
    }
}

/// Keeps all source/binary RGB contract data, stash etc
#[derive(Debug, Display)]
#[display_from(Debug)]
pub struct FileCache {
    config: FileCacheConfig,
    assets: HashMap<ContractId, Asset>,
}

impl FileCache {
    pub fn new(config: FileCacheConfig) -> Result<Self, FileCacheError> {
        debug!("Instantiating RGB fungible assets storage (disk storage) ...");

        let data_dir = config.data_dir.clone();
        if !data_dir.exists() {
            debug!(
                "RGB fungible assets data directory '{:?}' is not found; creating one",
                data_dir
            );
            fs::create_dir_all(data_dir)?;
        }
        let assets_dir = config.assets_dir();
        if !assets_dir.exists() {
            debug!(
                "RGB fungible assets information directory '{:?}' is not found; creating one",
                assets_dir
            );
            fs::create_dir_all(assets_dir)?;
        }

        let mut me = Self {
            config,
            assets: map![],
        };
        let filename = me.config.assets_filename();
        if filename.exists() {
            me.load()?;
        } else {
            debug!("Initializing assets file {:?} ...", filename.to_str());
            me.save()?;
        }

        Ok(me)
    }

    fn load(&mut self) -> Result<(), FileCacheError> {
        debug!("Reading assets information ...");
        let filename = self.config.assets_filename();
        let mut f = file(filename, FileMode::Read)?;
        self.assets = match self.config.data_format {
            DataFormat::Yaml => serde_yaml::from_reader(&f)?,
            DataFormat::Json => serde_json::from_reader(&f)?,
            DataFormat::Toml => {
                let mut data = String::new();
                f.read_to_string(&mut data)?;
                toml::from_str(&data)?
            }
            DataFormat::StrictEncode => unimplemented!(),
        };
        Ok(())
    }

    pub fn save(&self) -> Result<(), FileCacheError> {
        trace!("Saving assets information ...");
        let filename = self.config.assets_filename();
        let mut f = file(filename, FileMode::Create)?;
        match self.config.data_format {
            DataFormat::Yaml => serde_yaml::to_writer(&f, &self.assets)?,
            DataFormat::Json => serde_json::to_writer(&f, &self.assets)?,
            DataFormat::Toml => f.write_all(&toml::to_vec(&self.assets)?)?,
            DataFormat::StrictEncode => unimplemented!(),
        }
        Ok(())
    }

    pub fn export(&self) -> Result<Vec<u8>, FileCacheError> {
        trace!("Exporting assets information ...");
        let assets = self.assets.values().collect::<Vec<&Asset>>();
        Ok(match self.config.data_format {
            DataFormat::Yaml => serde_yaml::to_vec(&assets)?,
            DataFormat::Json => serde_json::to_vec(&assets)?,
            DataFormat::Toml => toml::to_vec(&assets)?,
            DataFormat::StrictEncode => unimplemented!(),
        })
    }
}

impl Cache for FileCache {
    type Error = CacheError;

    fn assets(&self) -> Result<Vec<&Asset>, CacheError> {
        Ok(self.assets.values().collect())
    }

    #[inline]
    fn asset(&self, id: ContractId) -> Result<&Asset, CacheError> {
        Ok(self.assets.get(&id).ok_or(CacheError::DataIntegrityError(
            "Asset is not known".to_string(),
        ))?)
    }

    #[inline]
    fn has_asset(&self, id: ContractId) -> Result<bool, CacheError> {
        Ok(self.assets.contains_key(&id))
    }

    fn add_asset(&mut self, asset: Asset) -> Result<bool, CacheError> {
        let exists = self.assets.insert(*asset.id(), asset).is_some();
        self.save()?;
        Ok(exists)
    }

    #[inline]
    fn remove_asset(&mut self, id: ContractId) -> Result<bool, CacheError> {
        Ok(self.assets.remove(&id).is_some())
    }
}
