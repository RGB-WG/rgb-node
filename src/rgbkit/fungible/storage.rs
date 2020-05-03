use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io, io::Read, io::Write};

use lnpbp::bitcoin;
use lnpbp::rgb::prelude::*;

use super::Asset;
use crate::rgbkit::{file::*, InteroperableError};

pub trait Store {
    fn assets(&self) -> Result<Vec<&Asset>, InteroperableError>;
    fn asset(&self, id: ContractId) -> Result<&Asset, InteroperableError>;
    fn has_asset(&self, id: ContractId) -> Result<bool, InteroperableError>;
    fn add_asset(&mut self, asset: Asset) -> Result<bool, InteroperableError>;
    fn remove_asset(&mut self, id: ContractId) -> Result<bool, InteroperableError>;
}

#[derive(Debug, Display, Error, From)]
#[display_from(Debug)]
pub enum DiskStorageError {
    #[derive_from]
    Io(io::Error),

    #[derive_from(bitcoin::hashes::Error)]
    HashName,

    #[derive_from]
    Encoding(lnpbp::strict_encoding::Error),

    #[derive_from(bitcoin::hashes::hex::Error)]
    BrokenHexFilenames,

    #[derive_from]
    SerdeJson(serde_json::Error),

    #[derive_from(std::option::NoneError)]
    NotFound,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display_from(Debug)]
pub struct DiskStorageConfig {
    pub data_dir: PathBuf,
}

impl DiskStorageConfig {
    pub const RGB_FA_EXTENSION: &'static str = "dat";

    #[inline]
    pub fn assets_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    #[inline]
    pub fn assets_filename(&self) -> PathBuf {
        self.assets_dir()
            .join("assets")
            .with_extension(Self::RGB_FA_EXTENSION)
    }
}

/// Keeps all source/binary RGB contract data, stash etc
#[derive(Debug, Display)]
#[display_from(Debug)]
pub struct DiskStorage {
    config: DiskStorageConfig,
    assets: HashMap<ContractId, Asset>,
}

impl DiskStorage {
    pub fn new(config: DiskStorageConfig) -> Result<Self, DiskStorageError> {
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

        debug!("Reading asset information ...");
        let filename = config.assets_filename();
        let assets = if filename.exists() {
            let mut f = file(filename, FileMode::Read)?;
            let mut contents = String::new();
            f.read_to_string(&mut contents)?;
            let assets = serde_json::from_str(&contents)?;
            assets
        } else {
            debug!("Initializing assets file {:?} ...", filename.to_str());
            let mut f = file(filename, FileMode::Create)?;
            let assets = HashMap::new();
            let data = serde_json::to_string(&assets)?;
            f.write_all(&data.as_bytes())?;
            assets
        };

        Ok(Self { config, assets })
    }

    pub fn save(&self) -> Result<(), DiskStorageError> {
        trace!("Saving updated asset information ...");
        let filename = self.config.assets_filename();
        let mut f = file(filename, FileMode::Create)?;
        let data = serde_json::to_string(&self.assets)?;
        f.write_all(&data.as_bytes())?;
        Ok(())
    }
}

impl Store for DiskStorage {
    fn assets(&self) -> Result<Vec<&Asset>, InteroperableError> {
        Ok(self.assets.values().collect())
    }

    #[inline]
    fn asset(&self, id: ContractId) -> Result<&Asset, InteroperableError> {
        Ok(self
            .assets
            .get(&id)
            .ok_or(InteroperableError(format!("Asset {} s not known", id)))?)
    }

    #[inline]
    fn has_asset(&self, id: ContractId) -> Result<bool, InteroperableError> {
        Ok(self.assets.contains_key(&id))
    }

    fn add_asset(&mut self, asset: Asset) -> Result<bool, InteroperableError> {
        let exists = self.assets.insert(asset.id(), asset).is_some();
        self.save()?;
        Ok(exists)
    }

    #[inline]
    fn remove_asset(&mut self, id: ContractId) -> Result<bool, InteroperableError> {
        Ok(self.assets.remove(&id).is_some())
    }
}
