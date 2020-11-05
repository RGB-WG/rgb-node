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
use lnpbp::rgb::prelude::*;
use lnpbp::strict_encoding::strict_encode;

use super::Cache;
use crate::fungible::cache::CacheError;
use crate::fungible::Asset;
use crate::util::file::*;
use crate::DataFormat;

#[derive(Debug, Display, Error, From)]
#[display(Debug)]
pub enum FileCacheError {
    #[from]
    Io(io::Error),

    #[from(lnpbp::hashes::Error)]
    HashName,

    #[from(lnpbp::hex::Error)]
    BrokenHexFilenames,

    #[from]
    Encoding(lnpbp::strict_encoding::Error),

    #[from]
    SerdeJson(serde_json::Error),

    #[from]
    SerdeYaml(serde_yaml::Error),

    #[from(toml::de::Error)]
    #[from(toml::ser::Error)]
    SerdeToml,

    NotFound,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(Debug)]
pub struct FileCacheConfig {
    pub data_dir: PathBuf,
    pub data_format: DataFormat,
}

impl FileCacheConfig {
    #[inline]
    pub fn assets_dir(&self) -> PathBuf {
        self.data_dir.join("assets")
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
#[display(Debug)]
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
        let _ = fs::remove_file(&filename);
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
        let assets = self.assets.values().cloned().collect::<Vec<Asset>>();
        Ok(match self.config.data_format {
            DataFormat::Yaml => serde_yaml::to_vec(&assets)?,
            DataFormat::Json => serde_json::to_vec(&assets)?,
            DataFormat::Toml => toml::to_vec(&assets)?,
            DataFormat::StrictEncode => strict_encode(&assets)?,
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
        let existed = self.assets.remove(&id).is_some();
        self.save()?;
        Ok(existed)
    }

    fn asset_allocations(
        &self,
        contract_id: ContractId,
    ) -> Result<HashMap<bitcoin::OutPoint, Vec<AtomicValue>>, CacheError> {
        // Process known_allocation map to produce the intended map
        let result: HashMap<bitcoin::OutPoint, Vec<AtomicValue>> = self
            .asset(contract_id)?
            .known_allocations()
            .into_iter()
            .map(|(outpoint, allocations)| {
                (
                    *outpoint,
                    allocations.into_iter().map(|a| a.value().value).collect(),
                )
            })
            .collect();

        Ok(result)
    }

    fn output_assets(
        &self,
        utxo: &bitcoin::OutPoint,
    ) -> Result<HashMap<ContractId, Vec<AtomicValue>>, CacheError> {
        let mut result = HashMap::new();

        for asset in self.assets()?.into_iter().cloned().collect::<Vec<Asset>>()
        {
            let allocations: Vec<AtomicValue> =
                match asset.known_allocations().get(utxo) {
                    Some(allocations) => allocations
                        .into_iter()
                        .map(|alloc| alloc.value().value)
                        .collect(),
                    None => continue,
                };

            result.insert(*asset.id(), allocations);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::super::sql::{SqlCache, SqlCacheConfig};
    use super::*;
    use lnpbp::hex::FromHex;
    use std::env;

    #[test]
    #[ignore]
    fn test_filecache_mappings() {
        // -------------------------------------------------
        // Setup sqlite database connection
        // This is done to easily copy test assets data
        // from sqlite database to required file format
        // As a consequence this should be run after running tests
        // in sql.rs module.

        let database_url = env::var("DATABASE_URL").expect(
            "Environment Variable 'DATABASE_URL' must be set to run this test",
        );
        let filepath = PathBuf::from(&database_url[..]);
        let config = SqlCacheConfig {
            data_dir: filepath.clone(),
        };
        let sql_cache = SqlCache::new(&config).unwrap();

        // Get the test assets
        let assets = sql_cache.assets().unwrap();

        // Setup a new Filecache with same Pathbuf and Json extension from above
        let filecache_config = FileCacheConfig {
            data_dir: filepath.clone(),
            data_format: DataFormat::Json,
        };

        // Init new FileCache
        let mut filecache = FileCache::new(filecache_config).unwrap();

        // Save the test assets inside FileCache
        // You should see an assets.json file in the DataDir
        for asset in assets {
            filecache.add_asset(asset.clone()).unwrap();
        }

        // -------------------------------------------------
        // TEST UTXO-ALLOCATION MAP

        // Test Contract_id to fetch data against
        let contract_id = ContractId::from_hex(
            "5bb162c7c84fa69bd263a12b277b82155787a03537691619fed731432f6855dc",
        )
        .unwrap();

        // Construct expected allocation-utxo mapping for the given asset
        // associated with the above contract_id
        let mut expected_map = HashMap::new();

        expected_map.insert(
            bitcoin::OutPoint {
                txid: bitcoin::Txid::from_hex(
                    "db2f3035e05795d72e2744dc0e88b2f72acbed97ee9a54c2c7f52d426ae05627",
                )
                .unwrap(),
                vout: 4,
            },
            vec![7 as AtomicValue, 9],
        );

        expected_map.insert(
            bitcoin::OutPoint {
                txid: bitcoin::Txid::from_hex(
                    "d47df6cf7a0eff79d3afeab7614404e43a0fa4498ff081918a2e75d7366cd730",
                )
                .unwrap(),
                vout: 5,
            },
            vec![11 as AtomicValue, 13],
        );

        expected_map.insert(
            bitcoin::OutPoint {
                txid: bitcoin::Txid::from_hex(
                    "fc63f797af718cc5a11988f69507701d5fe84e58cdd900e1b02856c0ea5a058a",
                )
                .unwrap(),
                vout: 3,
            },
            vec![1 as AtomicValue, 3, 5],
        );

        // Fetch the allocation-utxo map using cache api
        let calculated_map = filecache.asset_allocations(contract_id).unwrap();

        // Assert calculation meets expectation
        assert_eq!(expected_map, calculated_map);

        //-----------------------------------------------------
        // TEST ASSET-ALLOCATION MAP

        // Test utxo against which Asset allocations to be found
        let utxo = bitcoin::OutPoint {
            txid: bitcoin::Txid::from_hex(
                "fc63f797af718cc5a11988f69507701d5fe84e58cdd900e1b02856c0ea5a058a",
            )
            .unwrap(),
            vout: 3,
        };

        // Construct the expected mapping. The above utxo holds allocation
        // for 2 assets, Bitcoin and Ethereum. The target map is Map[Asset_name,
        // Allocated_amount]
        let mut expected_map = HashMap::new();
        expected_map.insert(ContractId::from_hex("5bb162c7c84fa69bd263a12b277b82155787a03537691619fed731432f6855dc").unwrap(), vec![1 as AtomicValue, 3, 5]);
        expected_map.insert(ContractId::from_hex("7ce3b67036e32628fe5351f23d57186181dba3103b7e0a5d55ed511446f5a6a9").unwrap(), vec![15 as AtomicValue, 17]);

        // Fetch the asset-amount map for the above utxo using cache api
        let allocation_map_calculated = filecache.output_assets(&utxo).unwrap();

        // Assert caclulation meets expectation
        assert_eq!(expected_map, allocation_map_calculated);
    }
}
