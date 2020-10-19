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

use diesel::prelude::*;
use std::collections::HashMap;
use std::{fmt, fs, fs::File};

use crate::contracts::fungible::cache::schema as cache_schema;

use lnpbp::bitcoin;
use lnpbp::bitcoin_hashes::hex::FromHex;
use lnpbp::rgb::ContractId;

use cache_schema::sql_allocation_utxo::dsl::sql_allocation_utxo as sql_allocation_utxo_table;
use cache_schema::sql_allocations::dsl::sql_allocations as sql_allocation_table;
use cache_schema::sql_assets::dsl::sql_assets as sql_asset_table;
use cache_schema::sql_inflation::dsl::sql_inflation as sql_inflation_table;
use cache_schema::sql_issues::dsl::sql_issues as sql_issue_table;

use crate::contracts::fungible::data::Asset;

use std::path::PathBuf;

use super::cache::{Cache, CacheError};

use super::models::*;

#[derive(Debug, Display, Error, From)]
#[display(Debug)]
pub enum SqlCacheError {
    #[from]
    Io(std::io::Error),

    #[from]
    Sqlite(diesel::result::Error),

    #[from(bitcoin::hashes::hex::Error)]
    HexDecoding,

    #[from]
    Generic(String),

    #[from]
    BlindKey(lnpbp::secp256k1zkp::Error),

    #[from]
    WrongChainData(lnpbp::bp::chain::ParseError),

    #[from(std::option::NoneError)]
    NotFound,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(Debug)]
pub struct SqlCacheConfig {
    pub data_dir: PathBuf,
}

impl SqlCacheConfig {
    #[inline]
    pub fn assets_dir(&self) -> PathBuf {
        self.data_dir.join("assets")
    }

    #[inline]
    pub fn assets_filename(&self) -> PathBuf {
        self.assets_dir().join("assets").with_extension("db")
    }
}

/// Keeps all source/binary RGB contract data, stash etc
pub struct SqlCache {
    connection: SqliteConnection,
    assets: HashMap<ContractId, Asset>,
}

impl fmt::Display for SqlCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self.assets)
    }
}

impl SqlCache {
    pub fn new(config: &SqlCacheConfig) -> Result<Self, SqlCacheError> {
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

        // check for cached db file
        let filename = config.assets_filename();

        if filename.exists() {
            // Create connection to db
            let connection = SqliteConnection::establish(config.assets_filename().to_str()?)
                .expect(&format!("Error connecting to asset.db"));

            let mut sql_cache = Self {
                connection,
                assets: map![],
            };

            sql_cache.load()?;

            Ok(sql_cache)
        } else {
            // If cached database not found create one and return empty cache
            debug!("Initializing assets file {:?} ...", filename.to_str());
            File::create(config.assets_filename())?;

            // Create connection to db
            let connection = SqliteConnection::establish(config.assets_filename().to_str()?)
                .expect(&format!("Error connecting to asset.db"));

            let sql_cache = Self {
                connection,
                assets: map![],
            };

            Ok(sql_cache)
        }
    }

    pub fn load(&mut self) -> Result<(), SqlCacheError> {
        // get the assets recorded in db
        let assets = sql_asset_table.load::<SqlAsset>(&self.connection)?;

        let mut asset_map = HashMap::new();

        for asset in assets {
            asset_map.insert(
                ContractId::from_hex(&asset.contract_id[..])?,
                Asset::from_sql_asset(&asset, &self.connection)?,
            );
        }

        self.assets = asset_map;

        Ok(())
    }

    /// Deletes and recreates the full database with updated cache
    pub fn save(&self) -> Result<(), SqlCacheError> {
        // Delet the existing data
        diesel::delete(sql_asset_table).execute(&self.connection)?;
        diesel::delete(sql_issue_table).execute(&self.connection)?;
        diesel::delete(sql_inflation_table).execute(&self.connection)?;
        diesel::delete(sql_allocation_utxo_table).execute(&self.connection)?;
        diesel::delete(sql_allocation_table).execute(&self.connection)?;

        // Create and write table entries from updated cached data
        for item in self.assets.clone().into_iter() {
            let table_asset = SqlAsset::from_asset(&item.1, &self.connection)?;
            let table_issues = SqlIssue::from_asset(&item.1, &table_asset, &self.connection)?;

            let table_inflations =
                SqlInflation::from_asset(&item.1, &table_asset, &self.connection)?;

            let (table_utxos, table_allocations) =
                create_allocation_from_asset(&item.1, &table_asset, &self.connection)?;

            diesel::insert_into(sql_asset_table)
                .values(table_asset)
                .execute(&self.connection)?;

            for issue in table_issues {
                diesel::insert_into(sql_issue_table)
                    .values(issue)
                    .execute(&self.connection)?;
            }

            for inflation in table_inflations {
                diesel::insert_into(sql_inflation_table)
                    .values(inflation)
                    .execute(&self.connection)?;
            }

            for utxo in table_utxos {
                diesel::insert_into(sql_allocation_utxo_table)
                    .values(utxo)
                    .execute(&self.connection)?;
            }

            for allocation in table_allocations {
                diesel::insert_into(sql_allocation_table)
                    .values(allocation)
                    .execute(&self.connection)?;
            }
        }

        Ok(())
    }
}

impl Cache for SqlCache {
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::contracts::fungible::data::Asset;
    use chrono::NaiveDate;
    use lnpbp::bitcoin_hashes::hex::FromHex;
    use lnpbp::rgb::ContractId;

    use cache_schema::sql_allocation_utxo::dsl::sql_allocation_utxo as sql_allocation_utxo_table;
    use cache_schema::sql_allocations::dsl::sql_allocations as sql_allocation_table;
    use cache_schema::sql_assets::dsl::sql_assets as sql_asset_table;
    use cache_schema::sql_inflation::dsl::sql_inflation as sql_inflation_table;
    use cache_schema::sql_issues::dsl::sql_issues as sql_issue_table;
    use std::env;

    // The following tests are ignored by default, because they will break
    // Travis CI unless the build setup is updated (TODO). To run these tests,
    // 1. set an environment variable DATABASE_URL=~/.rgb.
    // 2. manually remove the ignore flag and run the rgb-node/test/test_db.sh.

    #[test]
    #[ignore]
    // Creates a sample table with sample asset data
    fn test_create_tables() {
        let database_url = env::var("DATABASE_URL")
            .expect("Environment Variable 'DATABASE_URL' must be set to run this test");

        let filepath = PathBuf::from(&database_url[..]);

        let config = SqlCacheConfig { data_dir: filepath };

        println!("{:#?}", config);

        let cache = SqlCache::new(&config).unwrap();

        let conn = cache.connection;
        // Assets
        for i in 0..2 {
            match i {
                0 => {
                    let asset = SqlAsset {
                        id: i,
                        contract_id:
                            "5bb162c7c84fa69bd263a12b277b82155787a03537691619fed731432f6855dc"
                                .to_string(),
                        ticker: "BTC".to_string(),
                        asset_name: "Bitcoin".to_string(),
                        asset_description: Some("I am satoshi".to_string()),
                        known_circulating_supply: 20000,
                        is_issued_known: Some(true),
                        max_cap: 20000,
                        chain: lnpbp::bp::Chain::Mainnet.to_string(),
                        fractional_bits: vec![0u8],
                        asset_date: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
                    };

                    diesel::insert_into(sql_asset_table)
                        .values(asset)
                        .execute(&conn)
                        .unwrap();
                }

                1 => {
                    let asset = SqlAsset {
                        id: i,
                        contract_id:
                            "7ce3b67036e32628fe5351f23d57186181dba3103b7e0a5d55ed511446f5a6a9"
                                .to_string(),
                        ticker: "ETH".to_string(),
                        asset_name: "Ethereum".to_string(),
                        asset_description: Some("I am Vitalik".to_string()),
                        known_circulating_supply: 10000,
                        is_issued_known: Some(true),
                        max_cap: 10000,
                        chain: lnpbp::bp::Chain::Testnet3.to_string(),
                        fractional_bits: vec![0u8],
                        asset_date: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
                    };

                    diesel::insert_into(sql_asset_table)
                        .values(asset)
                        .execute(&conn)
                        .unwrap();
                }

                _ => {}
            }
        }

        // Issues
        for i in 0..6 {
            match i {
                0..=2 => {
                    let issue = SqlIssue {
                        id: i,
                        sql_asset_id: 0,
                        node_id: "2242ee7a53d9a1b67867891e9e5f2b0b80db0b0e6a983dd5cdc6df947385b554"
                            .to_string(),
                        contract_id:
                            "8c6c655eec5a030abc95a3695d11e4fc13136a233a05caafb16b45b012648bc5"
                                .to_string(),
                        amount: 5000,
                        origin_txid: Some(
                            "7d167ebb0182ef02f3402d3e7b0233bf4de6f808330086a82d6dd1cf59c03ac0"
                                .to_string(),
                        ),
                        origin_vout: Some(5),
                    };

                    diesel::insert_into(sql_issue_table)
                        .values(issue)
                        .execute(&conn)
                        .unwrap();
                }

                3..=5 => {
                    let issue = SqlIssue {
                        id: i,
                        sql_asset_id: 1,
                        node_id: "282de45c37bf6f7070106b7cf8dddfb41f9c5133785ab680aec1347ad2fd670c"
                            .to_string(),
                        contract_id:
                            "35e518ea2b087ea72e57b10c9768838eb86be9a49c14c72a914eb797ffbdcda4"
                                .to_string(),
                        amount: 5000,
                        origin_txid: None,
                        origin_vout: None,
                    };

                    diesel::insert_into(sql_issue_table)
                        .values(issue)
                        .execute(&conn)
                        .unwrap();
                }

                _ => {}
            }
        }

        // Infation

        for i in 0..5 {
            match i {
                0..=1 => {
                    let inflation = SqlInflation {
                        id: i,
                        sql_asset_id: 0,
                        outpoint_txid: match i {
                            0 => Some(
                                "eca6748a4a7cca292aefee3ec8df5af02de80891adc58ee2f455c6a00a723cfe"
                                    .to_string(),
                            ),
                            1 => None,
                            _ => Some("Somethings wrong".to_string()),
                        },
                        outpoint_vout: match i {
                            0 => Some(5),
                            1 => None,
                            _ => None,
                        },
                        accounting_amount: 3000,
                    };
                    diesel::insert_into(sql_inflation_table)
                        .values(inflation)
                        .execute(&conn)
                        .unwrap();
                }

                2..=3 => {
                    let inflation = SqlInflation {
                        id: i,
                        sql_asset_id: 1,
                        outpoint_txid: match i {
                            2 => Some(
                                "a2b8d4924f03d7ce008c95c8f9c0365944c12d7898cc0453be5010176117c3da"
                                    .to_string(),
                            ),
                            3 => None,
                            _ => Some("Somethings wrong".to_string()),
                        },
                        outpoint_vout: match i {
                            2 => Some(5),
                            3 => None,
                            _ => None,
                        },
                        accounting_amount: 3000,
                    };
                    diesel::insert_into(sql_inflation_table)
                        .values(inflation)
                        .execute(&conn)
                        .unwrap();
                }

                _ => {}
            }
        }

        // Allocation outpoints
        for i in 0..5 {
            match i {
                0..=1 => {
                    let allocation_utxo = SqlAllocationUtxo {
                        id: i,
                        sql_asset_id: 0,
                        txid: "fc63f797af718cc5a11988f69507701d5fe84e58cdd900e1b02856c0ea5a058a"
                            .to_string(),
                        vout: i * 2,
                    };

                    diesel::insert_into(sql_allocation_utxo_table)
                        .values(allocation_utxo)
                        .execute(&conn)
                        .unwrap();
                }
                2..=3 => {
                    let allocation_utxo = SqlAllocationUtxo {
                        id: i,
                        sql_asset_id: 1,
                        txid: "fc63f797af718cc5a11988f69507701d5fe84e58cdd900e1b02856c0ea5a058a"
                            .to_string(),
                        vout: i * 2,
                    };

                    diesel::insert_into(sql_allocation_utxo_table)
                        .values(allocation_utxo)
                        .execute(&conn)
                        .unwrap();
                }
                _ => {}
            }
        }

        // Allocations
        for i in 0..8 {
            match i {
                0..=1 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 0,
                        node_id: "3854d4e041fc301afee0c91ac06451ac7c0a0b37d965172f693d421769a27e94"
                            .to_string(),
                        assignment_index: i * 3 - 1,
                        amount: 50000,
                        blinding:
                            "7c62d1e24a6e99e30743ff94e5d3f783efc1ab8016d342558802c7f56e06ac15"
                                .to_string(),
                    };

                    diesel::insert_into(sql_allocation_table)
                        .values(allocation)
                        .execute(&conn)
                        .unwrap();
                }

                2..=3 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 1,
                        node_id: "3854d4e041fc301afee0c91ac06451ac7c0a0b37d965172f693d421769a27e94"
                            .to_string(),
                        assignment_index: i * 3 - 1,
                        amount: 50000,
                        blinding:
                            "7c62d1e24a6e99e30743ff94e5d3f783efc1ab8016d342558802c7f56e06ac15"
                                .to_string(),
                    };

                    diesel::insert_into(sql_allocation_table)
                        .values(allocation)
                        .execute(&conn)
                        .unwrap();
                }

                4..=5 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 2,
                        node_id: "3854d4e041fc301afee0c91ac06451ac7c0a0b37d965172f693d421769a27e94"
                            .to_string(),
                        assignment_index: i * 3 - 1,
                        amount: 50000,
                        blinding:
                            "7c62d1e24a6e99e30743ff94e5d3f783efc1ab8016d342558802c7f56e06ac15"
                                .to_string(),
                    };

                    diesel::insert_into(sql_allocation_table)
                        .values(allocation)
                        .execute(&conn)
                        .unwrap();
                }

                6..=7 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 3,
                        node_id: "3854d4e041fc301afee0c91ac06451ac7c0a0b37d965172f693d421769a27e94"
                            .to_string(),
                        assignment_index: i * 3 - 1,
                        amount: 50000,
                        blinding:
                            "7c62d1e24a6e99e30743ff94e5d3f783efc1ab8016d342558802c7f56e06ac15"
                                .to_string(),
                    };

                    diesel::insert_into(sql_allocation_table)
                        .values(allocation)
                        .execute(&conn)
                        .unwrap();
                }

                _ => {}
            }
        }

        println!("Tables created");
    }

    #[test]
    #[ignore]
    fn test_asset_cache() {
        let database_url = env::var("DATABASE_URL")
            .expect("Environment Variable 'DATABASE_URL' must be set to run this test");

        let filepath = PathBuf::from(&database_url[..]);
        let config = SqlCacheConfig { data_dir: filepath };

        let mut cache = SqlCache::new(&config).unwrap();

        // Test fetching all assets
        let assets = cache.assets().unwrap();

        assert_eq!(assets.len(), 2);

        // Test fetching single assets
        let asset = cache
            .asset(
                ContractId::from_hex(
                    "7ce3b67036e32628fe5351f23d57186181dba3103b7e0a5d55ed511446f5a6a9",
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(asset.name(), &String::from("Ethereum"));
        assert_eq!(
            asset.description().clone().unwrap(),
            String::from("I am Vitalik")
        );

        let asset = cache
            .asset(
                ContractId::from_hex(
                    "5bb162c7c84fa69bd263a12b277b82155787a03537691619fed731432f6855dc",
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(asset.name(), &String::from("Bitcoin"));
        assert_eq!(
            asset.description().clone().unwrap(),
            String::from("I am satoshi")
        );

        // Test checking existance of asset
        assert!(cache
            .has_asset(
                ContractId::from_hex(
                    "7ce3b67036e32628fe5351f23d57186181dba3103b7e0a5d55ed511446f5a6a9"
                )
                .unwrap()
            )
            .unwrap());
        assert!(cache
            .has_asset(
                ContractId::from_hex(
                    "5bb162c7c84fa69bd263a12b277b82155787a03537691619fed731432f6855dc"
                )
                .unwrap()
            )
            .unwrap());

        // Test Adding Asset
        // Copy the first asset entry and change contract id
        let mut first_sql_asset = sql_asset_table
            .load::<SqlAsset>(&cache.connection)
            .unwrap()
            .first()
            .cloned()
            .unwrap();

        first_sql_asset.contract_id =
            String::from("9b9dc7065be8fe0a965f42dc4d64bd1e15aa56cf05d3c72fc472c55490936bb3");

        let new_asset = Asset::from_sql_asset(&first_sql_asset, &cache.connection).unwrap();

        assert!(cache.add_asset(new_asset).is_ok());

        let mut new_cache = SqlCache::new(&config).unwrap();

        assert_eq!(new_cache.assets().unwrap().len(), 3);
        assert_eq!(
            new_cache
                .asset(
                    ContractId::from_hex(
                        "9b9dc7065be8fe0a965f42dc4d64bd1e15aa56cf05d3c72fc472c55490936bb3"
                    )
                    .unwrap()
                )
                .unwrap()
                .name(),
            &String::from("Bitcoin")
        );

        // Test removing asset the new added asset
        assert!(new_cache
            .remove_asset(
                ContractId::from_hex(
                    "9b9dc7065be8fe0a965f42dc4d64bd1e15aa56cf05d3c72fc472c55490936bb3"
                )
                .unwrap()
            )
            .unwrap());

        let newer_cache = SqlCache::new(&config).unwrap();

        assert_eq!(newer_cache.assets().unwrap().len(), 2);
    }
}
