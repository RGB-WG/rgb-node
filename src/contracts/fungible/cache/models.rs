use ::std::collections::BTreeMap;

use crate::contracts::fungible::cache::schema as cache_schema;
use cache_schema::sql_allocation_utxo::dsl::sql_allocation_utxo as sql_allocation_utxo_table;
use cache_schema::sql_allocations::dsl::sql_allocations as sql_allocation_table;
use cache_schema::sql_assets::dsl::sql_assets as sql_asset_table;
use cache_schema::sql_inflation::dsl::sql_inflation as sql_inflation_table;
use cache_schema::sql_issues::dsl::sql_issues as sql_issue_table;
use cache_schema::*;

use super::sql::SqlCacheError;
use crate::contracts::fungible::data::{AccountingAmount, AccountingValue, Allocation, Asset};
use diesel::prelude::*;
use lnpbp::bitcoin::{OutPoint, Txid};
use lnpbp::bitcoin_hashes::hex::{FromHex, ToHex};
use lnpbp::bp::Chain;

/// All the sqlite table structures are defined here.
/// There are 5 tables namely Asset, Issue, Inflation, AllocationUtxo
/// and Allocation. The Asset is the major table, and all other tables
/// are associated with Asset by sql_asset_id field.

#[derive(Queryable, Insertable, Identifiable, Clone, Debug)]
#[table_name = "sql_assets"]
pub struct SqlAsset {
    pub id: i32,
    pub contract_id: String,
    pub ticker: String,
    pub asset_name: String,
    pub asset_description: Option<String>,
    pub known_circulating_supply: i64,
    pub is_issued_known: Option<bool>,
    pub max_cap: i64,
    pub chain: String,
    pub fractional_bits: Vec<u8>,
    pub asset_date: chrono::NaiveDateTime,
}

impl SqlAsset {
    /// Create an Sqlite Asset entry from a given Asset data structure.
    /// Note, only the metadata are written into the Asset table,
    /// All other data for the Asset are to be found from fetching other
    /// table entries associated with this Asset entry. So they are implicitly defined
    /// in the table schema.   
    pub fn from_asset(asset: &Asset, connection: &SqliteConnection) -> Result<Self, SqlCacheError> {
        // Find the last entry and increase index by 1
        let last_asset = sql_asset_table
            .load::<SqlAsset>(connection)?
            .last()
            .cloned();

        Ok(Self {
            id: match last_asset {
                Some(asset) => asset.id + 1,
                None => 0,
            },
            contract_id: asset.id().clone().to_hex(),
            ticker: asset.ticker().clone(),
            asset_name: asset.name().clone(),
            asset_description: asset.description().clone(),
            known_circulating_supply: asset.supply().known_circulating().accounting_value() as i64,
            is_issued_known: asset.supply().is_issued_known().clone(),
            max_cap: asset.supply().max_cap().accounting_value() as i64,
            chain: write_chain_to_table(asset.chain())?,
            fractional_bits: vec![asset.fractional_bits().clone()],
            asset_date: asset.date().clone(),
        })
    }
}

// TODO: Implement handling for other chain variants
pub fn read_chain_from_table(table_value: String) -> Result<Chain, SqlCacheError> {
    match &table_value[..] {
        "MainNet" => Ok(Chain::Mainnet),
        "TestNet3" => Ok(Chain::Testnet3),
        _ => Err(SqlCacheError::GenericError(
            "Unsupported Chain value".to_string(),
        )),
    }
}

pub fn write_chain_to_table(chain: &Chain) -> Result<String, SqlCacheError> {
    match chain {
        Chain::Mainnet => Ok(String::from("MainNet")),
        Chain::Testnet3 => Ok(String::from("TestNet3")),
        _ => Err(SqlCacheError::GenericError(
            "Unsupported Chain value".to_string(),
        )),
    }
}

#[derive(Queryable, Insertable, Identifiable, Associations, Clone, Debug)]
#[table_name = "sql_inflation"]
#[belongs_to(SqlAsset)]
/// There is no separate field for unknown inflation amount in the db.
/// An Inflation structure with None type outpoint is used
/// as unknown inflation value for the associated asset.
pub struct SqlInflation {
    pub id: i32,
    pub sql_asset_id: i32,
    pub outpoint_txid: Option<String>,
    pub outpoint_vout: Option<i32>,
    pub accounting_amount: i64,
}

impl SqlInflation {
    pub fn from_asset(
        asset: &Asset,
        table_asset: &SqlAsset,
        connection: &SqliteConnection,
    ) -> Result<Vec<Self>, SqlCacheError> {
        // Find the last inflation entry and increase id from there
        let last_inflation = sql_inflation_table
            .load::<SqlInflation>(connection)?
            .last()
            .cloned();

        let known_inflations = asset.known_inflation();

        let mut result = vec![];

        // Create Inflation table entries from Asset known_inflation data
        for (index, item) in known_inflations.into_iter().enumerate() {
            let sql_inflation = Self {
                id: match last_inflation.clone() {
                    Some(inflation) => inflation.id + index as i32 + 1,
                    None => 0 + index as i32,
                },
                sql_asset_id: table_asset.id,
                outpoint_txid: Some(item.0.txid.to_hex()),
                outpoint_vout: Some(item.0.vout as i32),
                accounting_amount: item.1.accounting_value() as i64,
            };

            result.push(sql_inflation);
        }

        // We need to keep track on the last added item id
        // in the above inflation entry as we need to add
        // unknown inflation entry next.
        let last_added_id;

        if let Some(item) = result.last() {
            last_added_id = item.id;
        } else {
            last_added_id = 0;
        }

        // Push the unknown inflation entry with txid and vout as None.
        result.push(Self {
            id: last_added_id + 1,
            sql_asset_id: table_asset.id,
            outpoint_txid: None,
            outpoint_vout: None,
            accounting_amount: asset.unknown_inflation().accounting_value() as i64,
        });

        Ok(result)
    }
}

/// Read Inflation data associated to a table Asset entry
/// found at a given database connection
/// This returns the known_inflation and unknown inflation data structures
/// for the given asset in tupple. Which then can be directly used as fields
/// in Asset data structure.
pub fn read_inflation(
    asset: &SqlAsset,
    connection: &SqliteConnection,
) -> Result<(BTreeMap<OutPoint, AccountingAmount>, AccountingAmount), SqlCacheError> {
    let inflations = SqlInflation::belonging_to(asset).load::<SqlInflation>(connection)?;

    let mut known_inflation_map = BTreeMap::new();

    let mut unknown = AccountingAmount::default();

    for known_inflation in inflations {
        match (known_inflation.outpoint_txid, known_inflation.outpoint_vout) {
            // If both txid and vout are present add them to known_inflation_map
            (Some(txid), Some(vout)) => {
                known_inflation_map.insert(
                    OutPoint {
                        txid: Txid::from_hex(&txid[..])?,
                        vout: vout as u32,
                    },
                    AccountingAmount::from_fractioned_accounting_value(
                        asset.fractional_bits[0],
                        known_inflation.accounting_amount as AccountingValue,
                    ),
                );
            }
            // For everything else, add them to unknown inflation
            _ => {
                unknown = AccountingAmount::from_fractioned_accounting_value(
                    asset.fractional_bits[0],
                    known_inflation.accounting_amount as AccountingValue,
                );
            }
        }
    }

    Ok((known_inflation_map, unknown))
}

#[derive(Queryable, Insertable, Identifiable, Associations, Clone, Debug)]
#[table_name = "sql_issues"]
#[belongs_to(SqlAsset)]
pub struct SqlIssue {
    pub id: i32,
    pub sql_asset_id: i32,
    pub node_id: String,
    pub contract_id: String,
    pub amount: i64,
    pub origin_txid: Option<String>,
    pub origin_vout: Option<i32>,
}

impl SqlIssue {
    /// Create vectro of Issue table entries from a given Asset data
    pub fn from_asset(
        asset: &Asset,
        table_asset: &SqlAsset,
        connection: &SqliteConnection,
    ) -> Result<Vec<Self>, SqlCacheError> {
        // get the last issue and increase id from there
        let last_issue = sql_issue_table
            .load::<SqlIssue>(connection)?
            .last()
            .cloned();

        let asset_issues = asset.known_issues();

        let mut result = vec![];

        for (index, issue) in asset_issues.into_iter().enumerate() {
            result.push(Self {
                id: match last_issue.clone() {
                    Some(issue) => issue.id + index as i32 + 1,
                    None => 0 + index as i32,
                },
                sql_asset_id: table_asset.id,
                node_id: issue.id().to_hex(),
                contract_id: issue.asset_id().to_hex(),
                amount: issue.amount().accounting_value() as i64,
                origin_txid: match issue.origin() {
                    Some(outpoint) => Some(outpoint.txid.to_hex()),
                    None => None,
                },
                origin_vout: match issue.origin() {
                    Some(outpoint) => Some(outpoint.vout as i32),
                    None => None,
                },
            })
        }
        Ok(result)
    }
}

/// AllocationUtxo and Allocation are associated tables with each other.
/// Every AllocationUtxo is associated with an Asset entry.
/// Every Allocation is associated with an AllocationUtxo.
/// Together these two tables represent the `known_allocations` field
/// in the Asset data structure.
#[derive(
    Queryable, Insertable, Identifiable, Associations, Clone, Debug, Ord, PartialOrd, Eq, PartialEq,
)]
#[table_name = "sql_allocation_utxo"]
#[belongs_to(SqlAsset)]
pub struct SqlAllocationUtxo {
    pub id: i32,
    pub sql_asset_id: i32,
    pub txid: String,
    pub vout: i32,
}

#[derive(Queryable, Insertable, Identifiable, Associations, Clone, Debug)]
#[table_name = "sql_allocations"]
#[belongs_to(SqlAllocationUtxo)]
pub struct SqlAllocation {
    pub id: i32,
    pub sql_allocation_utxo_id: i32,
    pub node_id: String,
    pub assignment_index: i32,
    pub amount: i64,
    pub blinding: String,
}

/// Create a list of AllocationUtxo and Allocation table entry
/// with correct associations between them.
/// The AllocationUtxo entries are associated with the given Asset entry.
///
/// For a Given Assets structure, this will create the correct AllocationUtxo and
/// Allocation table entries to write to the database
pub fn create_allocation_from_asset(
    asset: &Asset,
    table_asset: &SqlAsset,
    connection: &SqliteConnection,
) -> Result<(Vec<SqlAllocationUtxo>, Vec<SqlAllocation>), SqlCacheError> {
    // get the last allocationutxo and allocation id
    // increase id from there
    let last_alloc_utxo = sql_allocation_utxo_table
        .load::<SqlAllocationUtxo>(connection)?
        .last()
        .cloned();
    let last_alloc = sql_allocation_table
        .load::<SqlAllocation>(connection)?
        .last()
        .cloned();

    let allocations = asset.known_allocations();

    let mut utxos = vec![];
    let mut allocation_vec = vec![];

    let mut added_allocations = 0;

    for (index, item) in allocations.into_iter().enumerate() {
        let this_utxo_id = match last_alloc_utxo.clone() {
            Some(alloc_utxo) => alloc_utxo.id + index as i32 + 1,
            None => 0 + index as i32,
        };
        utxos.push(SqlAllocationUtxo {
            id: this_utxo_id,
            sql_asset_id: table_asset.id,
            txid: item.0.txid.to_hex(),
            vout: item.0.vout as i32,
        });
        for (index, alloc) in item.1.into_iter().enumerate() {
            allocation_vec.push(SqlAllocation {
                id: match last_alloc.clone() {
                    Some(alloc) => alloc.id + added_allocations + index as i32 + 1,
                    None => 0 + added_allocations + index as i32,
                },
                sql_allocation_utxo_id: this_utxo_id,
                node_id: alloc.node_id().to_hex(),
                assignment_index: alloc.index().clone() as i32,
                amount: alloc.value().value as i64,
                blinding: alloc.value().blinding.0.to_vec().to_hex(),
            });
        }
        added_allocations += item.1.len() as i32;
    }

    Ok((utxos, allocation_vec))
}

/// Read the associated AllocationUtxo and Allocation entries
/// with the given Asset entry executed over the given database
/// connection.
/// This will return a BtreeMap which can be directly used as the
/// `known_allocations` field in the Asset Data structure.
pub fn read_allocations(
    asset: &SqlAsset,
    connection: &SqliteConnection,
) -> Result<BTreeMap<OutPoint, Vec<Allocation>>, SqlCacheError> {
    // Get the associated utxo with asset entry
    let utxo_list = SqlAllocationUtxo::belonging_to(asset).load::<SqlAllocationUtxo>(connection)?;

    // Get the associated allocations with the above utxos
    let allocations = SqlAllocation::belonging_to(&utxo_list)
        .load::<SqlAllocation>(connection)?
        .grouped_by(&utxo_list);

    // Group them accordingly and zip the two vectors by correct groupings
    let grouped_allocation = utxo_list.into_iter().zip(&allocations).collect::<Vec<_>>();

    let mut allocation_map = BTreeMap::new();

    grouped_allocation.into_iter().for_each(|item| {
        allocation_map.insert(item.0, item.1);
    });

    // Read the zipped data into a BtreeMap
    let mut known_allocation = BTreeMap::new();

    for item in allocation_map.into_iter() {
        let mut allocations = vec![];
        for allocation in item.1 {
            allocations.push(Allocation::from_sql_allocation(&allocation, &item.0)?);
        }
        known_allocation.insert(
            OutPoint {
                txid: Txid::from_hex(&item.0.txid[..])?,
                vout: item.0.vout as u32,
            },
            allocations,
        );
    }

    Ok(known_allocation)
}
