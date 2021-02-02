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
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::str::FromStr;
use std::{fmt, fs, fs::File};

use amplify::IoError;
use bitcoin::hashes::hex::ToHex;
use rgb::bech32;
use rgb::prelude::*;
use rgb20::Asset;

use crate::fungibled::sql::schema as cache_schema;
use cache_schema::sql_allocation_utxo::dsl::sql_allocation_utxo as sql_allocation_utxo_table;
use cache_schema::sql_allocations::dsl::sql_allocations as sql_allocation_table;
use cache_schema::sql_assets::dsl::sql_assets as sql_asset_table;
use cache_schema::sql_inflation::dsl::sql_inflation as sql_inflation_table;
use cache_schema::sql_issues::dsl::sql_issues as sql_issue_table;

use super::cache::{Cache, CacheError};
use crate::fungibled::sql::models::*;

#[derive(Debug, Display, Error, From)]
#[display(inner)]
pub enum SqlCacheError {
    #[from(std::io::Error)]
    Io(IoError),

    #[from]
    Sqlite(diesel::result::Error),

    #[from]
    HexDecoding(bitcoin::hashes::hex::Error),

    #[from]
    Bech32(bech32::Error),

    #[from]
    Generic(String),

    #[from]
    BlindKey(rgb::secp256k1zkp::Error),

    #[from]
    WrongChainData(lnpbp::chain::ParseError),

    #[display("Item not found")]
    NotFound,
}

impl From<SqlCacheError> for CacheError {
    fn from(err: SqlCacheError) -> Self {
        match err {
            SqlCacheError::Io(e) => Self::Io(format!("{:?}", e)),
            SqlCacheError::Sqlite(e) => {
                Self::Sqlite(format!("Error from sqlite asset cache {}", e.to_string()))
            }
            SqlCacheError::HexDecoding(_) => Self::DataIntegrityError(format!(
                "Wrong hex encoded data in sqlite asset cache table"
            )),
            SqlCacheError::Generic(e) => Self::DataIntegrityError(e),
            SqlCacheError::WrongChainData(e) => Self::DataIntegrityError(format!(
                "Wrong Chain data in sqlite asset cache table: {}",
                e
            )),
            SqlCacheError::NotFound => {
                Self::DataIntegrityError(format!("Asset cache sqlite database file not found"))
            }
            SqlCacheError::BlindKey(e) => Self::DataIntegrityError(format!(
                "Wrong amount blinding factor in asset cache sqlite database: {}",
                e
            )),
            SqlCacheError::Bech32(e) => Self::DataIntegrityError(e.to_string()),
        }
    }
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
            let connection = SqliteConnection::establish(
                config
                    .assets_filename()
                    .to_str()
                    .ok_or(SqlCacheError::NotFound)?,
            )
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
            let connection = SqliteConnection::establish(
                config
                    .assets_filename()
                    .to_str()
                    .ok_or(SqlCacheError::NotFound)?,
            )
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
                ContractId::from_str(&asset.contract_id[..])?,
                asset.to_asset(&self.connection)?,
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
            let table_issues =
                SqlIssue::from_asset(&item.1, &table_asset, &self.connection)?;

            let table_inflations = SqlInflation::from_asset(
                &item.1,
                &table_asset,
                &self.connection,
            )?;

            let (table_utxos, table_allocations) =
                create_allocation_from_asset(
                    &item.1,
                    &table_asset,
                    &self.connection,
                )?;

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

    // TODO: Move this method to RGB20
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
        // Explicitly local import
        // Will cause name clash in global scope otherwise
        use cache_schema::sql_allocation_utxo::dsl::*;

        // fetch all the utxo data from sqlite table matching the txid and vout
        // There can be multiple utxos having same (txid,vout) pair but linked
        // with different assets
        let sql_utxos = sql_allocation_utxo
            .filter(txid.eq(outpoint.txid.to_hex()))
            .filter(vout.eq(outpoint.vout as i32))
            .load::<SqlAllocationUtxo>(&self.connection)
            .map_err(|e| SqlCacheError::Sqlite(e))?;

        // fetch all allocation data corresponding to the utxos
        // and group them by their linked utxo
        let allocations = SqlAllocation::belonging_to(&sql_utxos)
            .load::<SqlAllocation>(&self.connection)
            .map_err(|e| SqlCacheError::Sqlite(e))?
            .grouped_by(&sql_utxos);

        // Create a vector group of Vec<(Utxo, Vec<Allocation>)> for easier
        // processing
        let utxo_allocation_groups =
            sql_utxos.into_iter().zip(allocations).collect::<Vec<_>>();

        // Create the empty result map
        let mut result = BTreeMap::new();

        // Process the above groups to produce the required map
        for (utxo, allocations) in utxo_allocation_groups {
            let contract_id_string = sql_asset_table
                .find(utxo.sql_asset_id)
                .first::<SqlAsset>(&self.connection)
                .map_err(|e| SqlCacheError::Sqlite(e))?
                .contract_id
                .clone();

            let contract_id = ContractId::from_str(&contract_id_string[..])
                .map_err(|e| SqlCacheError::Bech32(e))?;

            let allocs: Vec<AtomicValue> = allocations
                .iter()
                .map(|alloc| alloc.amount as AtomicValue)
                .collect();

            result.insert(contract_id, allocs);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bitcoin::hashes::hex::FromHex;
    use chrono::NaiveDate;
    use lnpbp::{Chain, TaggedHash};
    use rgb::ContractId;
    use std::env;

    use cache_schema::sql_allocation_utxo::dsl::sql_allocation_utxo as sql_allocation_utxo_table;
    use cache_schema::sql_allocations::dsl::sql_allocations as sql_allocation_table;
    use cache_schema::sql_assets::dsl::sql_assets as sql_asset_table;
    use cache_schema::sql_inflation::dsl::sql_inflation as sql_inflation_table;
    use cache_schema::sql_issues::dsl::sql_issues as sql_issue_table;

    // The following tests are ignored by default, because they will break
    // Travis CI unless the build setup is updated (TODO). To run these tests,
    // 1. set an environment variable DATABASE_URL=~/.rgb.
    // 2. manually remove the ignore flag and run the rgb-node/test/test_db.sh.

    #[test]
    #[ignore]
    // Creates a sample table with sample asset data
    fn test_sqlite_create_tables() {
        let database_url = env::var("DATABASE_URL").expect(
            "Environment Variable 'DATABASE_URL' must be set to run this test",
        );

        let filepath = PathBuf::from(&database_url[..]);

        let config = SqlCacheConfig { data_dir: filepath };

        let cache = SqlCache::new(&config).unwrap();

        let conn = cache.connection;
        // Assets
        for i in 0..2 {
            match i {
                0 => {
                    let asset = SqlAsset {
                        genesis: "".to_string(),
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
                        chain: Chain::Mainnet.to_string(),
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
                        genesis: "".to_string(),
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
                        chain: Chain::Testnet3.to_string(),
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
        let utxo_1 = SqlAllocationUtxo {
            id: 0,
            sql_asset_id: 0,
            txid: "fc63f797af718cc5a11988f69507701d5fe84e58cdd900e1b02856c0ea5a058a".to_string(),
            vout: 3,
        };

        let utxo_2 = SqlAllocationUtxo {
            id: 1,
            sql_asset_id: 0,
            txid: "db2f3035e05795d72e2744dc0e88b2f72acbed97ee9a54c2c7f52d426ae05627".to_string(),
            vout: 4,
        };

        let utxo_3 = SqlAllocationUtxo {
            id: 2,
            sql_asset_id: 0,
            txid: "d47df6cf7a0eff79d3afeab7614404e43a0fa4498ff081918a2e75d7366cd730".to_string(),
            vout: 5,
        };

        let utxo_1new = SqlAllocationUtxo {
            id: 3,
            sql_asset_id: 1,
            txid: "fc63f797af718cc5a11988f69507701d5fe84e58cdd900e1b02856c0ea5a058a".to_string(),
            vout: 3,
        };

        let utxo_4 = SqlAllocationUtxo {
            id: 4,
            sql_asset_id: 1,
            txid: "d9916ece4f595a2a3b58e3ba262d83e82fc33e410a22ed6959731c4ce1d8e7b0".to_string(),
            vout: 6,
        };

        let utxo_5 = SqlAllocationUtxo {
            id: 5,
            sql_asset_id: 1,
            txid: "0bde6052602fcadfeddbc2c4fe77ffc32a3c011a1a8a4c3ac11622e30c4d3363".to_string(),
            vout: 7,
        };

        let utxos = vec![utxo_1, utxo_2, utxo_3, utxo_1new, utxo_4, utxo_5];

        for utxo in utxos {
            diesel::insert_into(sql_allocation_utxo_table)
                .values(utxo)
                .execute(&conn)
                .unwrap();
        }

        // Allocations
        for i in 0..13 {
            match i {
                0..=2 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 0,
                        node_id: "3854d4e041fc301afee0c91ac06451ac7c0a0b37d965172f693d421769a27e94"
                            .to_string(),
                        assignment_index: i + 3,
                        amount: 2 * (i as i64) + 1,
                        blinding:
                            "7c62d1e24a6e99e30743ff94e5d3f783efc1ab8016d342558802c7f56e06ac15"
                                .to_string(),
                    };

                    diesel::insert_into(sql_allocation_table)
                        .values(allocation)
                        .execute(&conn)
                        .unwrap();
                }

                3..=4 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 1,
                        node_id: "ab92d4827105723ecbfdccb12f81a2f74e36b320de8ca55435ae8ae60e290994"
                            .to_string(),
                        assignment_index: i + 3,
                        amount: 2 * (i as i64) + 1,
                        blinding:
                            "d55723e84d9ac6f611610d04dfa3d4b32757d681e449201f9e587c1ecd7bcf78"
                                .to_string(),
                    };

                    diesel::insert_into(sql_allocation_table)
                        .values(allocation)
                        .execute(&conn)
                        .unwrap();
                }

                5..=6 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 2,
                        node_id: "cd1a3f69e9294d8feb9fb5a16ba5329325aaf24e647e4711b93ee80b4c1c8398"
                            .to_string(),
                        assignment_index: i + 3,
                        amount: 2 * (i as i64) + 1,
                        blinding:
                            "644549d3ac1349ec0143082b75d66b833be08b77d7e5f53c24a22ea9c16415fb"
                                .to_string(),
                    };

                    diesel::insert_into(sql_allocation_table)
                        .values(allocation)
                        .execute(&conn)
                        .unwrap();
                }

                7..=8 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 3,
                        node_id: "2ee1154c4015472a0b35f0a43c6f684cb103eb418705cdeae1567e30433e9a0b"
                            .to_string(),
                        assignment_index: i + 3,
                        amount: 2 * (i as i64) + 1,
                        blinding:
                            "37ec2ed7445ff79ca0ef3a2c95e404a4a65ba0e55c4e9e5ab26f1dde8eaa520b"
                                .to_string(),
                    };

                    diesel::insert_into(sql_allocation_table)
                        .values(allocation)
                        .execute(&conn)
                        .unwrap();
                }

                9..=10 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 4,
                        node_id: "86d8a32acdd82affc4f065a2894e0f9a036a3205f7cf4159d44fb211fa266cb1"
                            .to_string(),
                        assignment_index: i + 3,
                        amount: 2 * (i as i64) + 1,
                        blinding:
                            "e2c314fe21e23e1e349851c23b6c74a8de3e938af79fb31b4e521921980443c3"
                                .to_string(),
                    };

                    diesel::insert_into(sql_allocation_table)
                        .values(allocation)
                        .execute(&conn)
                        .unwrap();
                }

                11..=12 => {
                    let allocation = SqlAllocation {
                        id: i,
                        sql_allocation_utxo_id: 5,
                        node_id: "01396c8b312b0b9a46eed9b0b2bea9269bade59e4b6fc8883efe7fb62cd6e533"
                            .to_string(),
                        assignment_index: i + 3,
                        amount: 2 * (i as i64) + 1,
                        blinding:
                            "56e3d4561b3404353f3fd0f5729615f85e980f90a46b6a15192b8c4da97c6738"
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
    fn test_sqlite_asset_cache() {
        let database_url = env::var("DATABASE_URL").expect(
            "Environment Variable 'DATABASE_URL' must be set to run this test",
        );

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

        first_sql_asset.contract_id = String::from(
            "9b9dc7065be8fe0a965f42dc4d64bd1e15aa56cf05d3c72fc472c55490936bb3",
        );

        let new_asset = first_sql_asset.to_asset(&cache.connection).unwrap();

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

    #[test]
    #[ignore]
    fn test_sqlite_mappings() {
        //----------------------------------------------
        // SETUP SQLITE DB CONNECTION

        let database_url = env::var("DATABASE_URL").expect(
            "Environment Variable 'DATABASE_URL' must be set to run this test",
        );
        let filepath = PathBuf::from(&database_url[..]);
        let config = SqlCacheConfig { data_dir: filepath };
        let cache = SqlCache::new(&config).unwrap();

        //----------------------------------------------
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
        let calculated_map = cache.asset_allocations(contract_id).unwrap();

        // Assert calculation meets expectation
        assert_eq!(expected_map, calculated_map);

        //----------------------------------------------
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
        let allocation_map_calculated = cache.outpoint_assets(utxo).unwrap();

        // Assert caclulation meets expectation
        assert_eq!(expected_map, allocation_map_calculated);
    }
}
