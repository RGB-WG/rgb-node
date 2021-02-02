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

#[cfg(feature = "serde")]
use serde_json;
use std::collections::BTreeMap;
#[cfg(any(
    feature = "serde_yaml",
    feature = "serde_json",
    feature = "toml"
))]
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs, io};

use lnpbp::strict_encoding::{strict_serialize, StrictDecode, StrictEncode};
use rgb::prelude::*;
use rgb20::Asset;

use super::Cache;
use crate::fungibled::cache::CacheError;
use crate::util::file::*;
use microservices::FileFormat;

#[derive(Debug, Display, Error, From)]
#[display(Debug)]
#[non_exhaustive]
pub enum FileCacheError {
    #[from]
    Io(io::Error),

    #[from(bitcoin::hashes::Error)]
    HashName,

    #[from(bitcoin::hashes::hex::Error)]
    BrokenHexFilenames,

    #[from]
    Encoding(lnpbp::strict_encoding::Error),

    #[cfg(feature = "serde")]
    #[from]
    SerdeJson(serde_json::Error),

    #[cfg(feature = "serde")]
    #[from]
    SerdeYaml(serde_yaml::Error),

    #[cfg(feature = "serde")]
    #[from(toml::de::Error)]
    #[from(toml::ser::Error)]
    SerdeToml,

    NotFound,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(Debug)]
pub struct FileCacheConfig {
    pub data_dir: PathBuf,
    pub data_format: FileFormat,
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
#[derive(Debug)]
pub struct FileCache {
    config: FileCacheConfig,
    assets: BTreeMap<ContractId, Asset>,
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
            assets: bmap![],
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
            #[cfg(feature = "serde_yaml")]
            FileFormat::Yaml => serde_yaml::from_reader(&f)?,
            #[cfg(feature = "serde_json")]
            FileFormat::Json => serde_json::from_reader(&f)?,
            #[cfg(feature = "toml")]
            FileFormat::Toml => {
                let mut data = String::new();
                f.read_to_string(&mut data)?;
                toml::from_str(&data)?
            }
            FileFormat::StrictEncode => StrictDecode::strict_decode(&mut f)?,
            _ => unimplemented!(),
        };
        Ok(())
    }

    pub fn save(&self) -> Result<(), FileCacheError> {
        trace!("Saving assets information ...");
        let filename = self.config.assets_filename();
        let _ = fs::remove_file(&filename);
        let mut f = file(filename, FileMode::Create)?;
        match self.config.data_format {
            #[cfg(feature = "serde_yaml")]
            FileFormat::Yaml => serde_yaml::to_writer(&f, &self.assets)?,
            #[cfg(feature = "serde_json")]
            FileFormat::Json => serde_json::to_writer(&f, &self.assets)?,
            #[cfg(feature = "toml")]
            FileFormat::Toml => f.write_all(&toml::to_vec(&self.assets)?)?,
            FileFormat::StrictEncode => {
                self.assets.strict_encode(&mut f)?;
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    pub fn export(
        &self,
        data_format: Option<FileFormat>,
    ) -> Result<Vec<u8>, FileCacheError> {
        trace!("Exporting assets information ...");
        let assets = self.assets.values().cloned().collect::<Vec<Asset>>();
        Ok(match data_format.unwrap_or(self.config.data_format) {
            #[cfg(feature = "serde_yaml")]
            FileFormat::Yaml => serde_yaml::to_vec(&assets)?,
            #[cfg(feature = "serde_json")]
            FileFormat::Json => serde_json::to_vec(&assets)?,
            #[cfg(feature = "toml")]
            FileFormat::Toml => toml::to_vec(&assets)?,
            FileFormat::StrictEncode => strict_serialize(&assets)?,
            _ => unimplemented!(),
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
    ) -> Result<BTreeMap<bitcoin::OutPoint, Vec<AtomicValue>>, CacheError> {
        // Process known_allocation map to produce the intended map
        let mut result = BTreeMap::<bitcoin::OutPoint, Vec<AtomicValue>>::new();
        for allocation in self.asset(contract_id)?.known_allocations() {
            result
                .entry(*allocation.outpoint())
                .or_insert(default!())
                .push(allocation.confidential_amount().value);
        }
        Ok(result)
    }

    fn outpoint_assets(
        &self,
        outpoint: bitcoin::OutPoint,
    ) -> Result<BTreeMap<ContractId, Vec<AtomicValue>>, CacheError> {
        let mut result = BTreeMap::new();

        for asset in self.assets()? {
            result.insert(
                *asset.id(),
                asset
                    .allocations(&outpoint)
                    .into_iter()
                    .map(|a| a.confidential_amount().value)
                    .collect(),
            );
        }

        Ok(result)
    }
}

#[cfg(all(test, feature = "sql"))]
mod test {
    use super::super::sql::{SqlCache, SqlCacheConfig};
    use super::*;
    use bitcoin::hashes::hex::FromHex;
    use lnpbp::TaggedHash;
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
            #[cfg(feature = "serde_json")]
            data_format: FileFormat::Json,
            #[cfg(not(feature = "serde_json"))]
            data_format: FileFormat::StrictEncode,
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
        let mut expected_map = BTreeMap::new();

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
        let mut expected_map = BTreeMap::new();
        expected_map.insert(ContractId::from_hex("5bb162c7c84fa69bd263a12b277b82155787a03537691619fed731432f6855dc").unwrap(), vec![1 as AtomicValue, 3, 5]);
        expected_map.insert(ContractId::from_hex("7ce3b67036e32628fe5351f23d57186181dba3103b7e0a5d55ed511446f5a6a9").unwrap(), vec![15 as AtomicValue, 17]);

        // Fetch the asset-amount map for the above utxo using cache api
        let allocation_map_calculated =
            filecache.outpoint_assets(utxo).unwrap();

        // Assert caclulation meets expectation
        assert_eq!(expected_map, allocation_map_calculated);
    }
}
