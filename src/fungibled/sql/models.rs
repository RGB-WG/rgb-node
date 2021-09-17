use diesel::prelude::*;
use std::collections::BTreeMap;
use std::str::FromStr;

use bitcoin::hashes::hex::{FromHex, ToHex};
use bitcoin::{OutPoint, Txid};
use lnpbp::chain::Chain;
use rgb::{value, AtomicValue, ContractId, NodeId};
use rgb20::{Asset, Issue, Supply};
use rgb::Allocation;
use crate::fungibled::sql::schema as cache_schema;
use crate::fungibled::SqlCacheError;
use cache_schema::sql_allocation_utxo::dsl::sql_allocation_utxo as sql_allocation_utxo_table;
use cache_schema::sql_allocations::dsl::sql_allocations as sql_allocation_table;
use cache_schema::sql_assets::dsl::sql_assets as sql_asset_table;
use cache_schema::sql_inflation::dsl::sql_inflation as sql_inflation_table;
use cache_schema::sql_issues::dsl::sql_issues as sql_issue_table;
use cache_schema::*;
use rgb::contract::value::BlindingFactor;

/// All the sqlite table structures are defined here.
/// There are 5 tables namely Asset, Issue, Inflation, AllocationUtxo
/// and Allocation. The Asset is the major table, and all other tables
/// are associated with Asset by sql_asset_id field.

#[derive(Queryable, Insertable, Identifiable, Clone, Debug)]
#[table_name = "sql_assets"]
pub struct SqlAsset {
    pub genesis: String,
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
    /// table entries associated with this Asset entry. So they are implicitly
    /// defined in the table schema.   
    pub fn from_asset(
        asset: &Asset,
        connection: &SqliteConnection,
    ) -> Result<Self, SqlCacheError> {
        // Find the last entry and increase index by 1
        let last_asset = sql_asset_table
            .load::<SqlAsset>(connection)?
            .last()
            .cloned();

        Ok(Self {
            genesis: asset.genesis().clone(),
            id: match last_asset {
                Some(asset) => asset.id + 1,
                None => 0,
            },
            contract_id: asset.id().clone().to_hex(),
            ticker: asset.ticker().clone(),
            asset_name: asset.name().clone(),
            asset_description: asset.description().clone(),
            known_circulating_supply: *asset.supply().known_circulating()
                as i64,
            is_issued_known: asset.supply().is_issued_known().clone(),
            max_cap: *asset.supply().issue_limit() as i64,
            chain: asset.chain().to_string(),
            fractional_bits: vec![asset.decimal_precision().clone()],
            asset_date: asset.date().clone(),
        })
    }

    /// Creates an [`Asset`] structure from an sqlite asset table entry.
    /// This fetches all the other tables for entries associated with the given
    /// asset and recreates the full Asset structure. This should be used
    /// while reading Asset data from database table.
    pub fn to_asset(
        &self,
        connection: &SqliteConnection,
    ) -> Result<Asset, SqlCacheError> {
        let known_inflation = read_inflation(self, connection)?;

        let known_table_issues =
            SqlIssue::belonging_to(self).load::<SqlIssue>(connection)?;

        let mut known_issues = vec![];

        for issue in known_table_issues {
            known_issues.push(issue.to_issue()?)
        }

        Ok(Asset::with(
            self.genesis.clone(),
            ContractId::from_str(&self.contract_id[..])?,
            self.ticker.clone(),
            self.asset_name.clone(),
            self.asset_description.clone(),
            self.to_supply(),
            Chain::from_str(&self.chain[..])?,
            self.fractional_bits[0],
            self.asset_date,
            known_issues,
            known_inflation,
            read_allocations(&self, connection)?,
        ))
    }

    /// Supply do not have a corresponding table in the database.
    /// The concerned values are written in the Asset table itself.
    /// This reads up an asset table entry and create the corresponding
    /// supply structure.
    pub fn to_supply(&self) -> Supply {
        Supply::with(
            self.known_circulating_supply as AtomicValue,
            self.is_issued_known,
            self.max_cap as AtomicValue,
        )
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
                accounting_amount: *item.1 as i64,
            };

            result.push(sql_inflation);
        }

        /* Unknown inflation is not used anymore
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
            accounting_amount: *asset.unknown_inflation() as i64,
        });
         */

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
) -> Result<BTreeMap<OutPoint, AtomicValue>, SqlCacheError> {
    let inflations =
        SqlInflation::belonging_to(asset).load::<SqlInflation>(connection)?;

    let mut known_inflation_map = BTreeMap::new();

    for inflation in inflations {
        match (inflation.outpoint_txid, inflation.outpoint_vout) {
            // If both txid and vout are present add them to known_inflation_map
            (Some(txid), Some(vout)) => {
                known_inflation_map.insert(
                    OutPoint {
                        txid: Txid::from_hex(&txid[..])?,
                        vout: vout as u32,
                    },
                    inflation.accounting_amount as AtomicValue,
                );
            }
            _ => return Err(SqlCacheError::NotFound),
        }
    }

    Ok(known_inflation_map)
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
                contract_id: table_asset.contract_id.clone(),
                amount: *issue.amount() as i64,
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

    /// Create an Issue structure from reading the corresponding
    /// Issue table entry in the database.
    pub fn to_issue(&self) -> Result<Issue, SqlCacheError> {
        Ok(Issue::with(
            NodeId::from_hex(&self.node_id[..])?,
            self.amount as AtomicValue,
            match (&self.origin_txid, self.origin_vout) {
                (Some(txid), Some(vout)) => Some(OutPoint {
                    txid: Txid::from_hex(&txid[..])?,
                    vout: vout as u32,
                }),
                _ => None,
            },
        ))
    }
}

/// AllocationUtxo and Allocation are associated tables with each other.
/// Every AllocationUtxo is associated with an Asset entry.
/// Every Allocation is associated with an AllocationUtxo.
/// Together these two tables represent the `known_allocations` field
/// in the Asset data structure.
#[derive(
    Queryable,
    Insertable,
    Identifiable,
    Associations,
    Clone,
    Debug,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
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

impl SqlAllocation {
    /// Create an Allocation structure by reading the
    /// corresponding Allocation and AllocationUtxo table entries.
    pub fn to_allocation(
        &self,
        outpoint: &SqlAllocationUtxo,
    ) -> Result<Allocation, SqlCacheError> {
        Ok(Allocation::with(
            NodeId::from_hex(&self.node_id[..])?,
            self.assignment_index as u16,
            OutPoint {
                txid: Txid::from_hex(&outpoint.txid[..])?,
                vout: outpoint.vout as u32,
            },
            value::Revealed {
                value: self.amount as AtomicValue,
                blinding: BlindingFactor::from_hex(&self.blinding)?,
            },
        ))
    }
}

/// Create a list of AllocationUtxo and Allocation table entry
/// with correct associations between them.
/// The AllocationUtxo entries are associated with the given Asset entry.
///
/// For a Given Assets structure, this will create the correct AllocationUtxo
/// and Allocation table entries to write to the database
pub fn create_allocation_from_asset(
    asset: &Asset,
    table_asset: &SqlAsset,
    connection: &SqliteConnection,
) -> Result<(Vec<SqlAllocationUtxo>, Vec<SqlAllocation>), SqlCacheError> {
    // get the last allocationutxo and allocation id
    // increase id from there
    let last_allocation_utxo = sql_allocation_utxo_table
        .load::<SqlAllocationUtxo>(connection)?
        .last()
        .cloned();
    let last_allocation = sql_allocation_table
        .load::<SqlAllocation>(connection)?
        .last()
        .cloned();
    let utxo_offset = last_allocation_utxo.map(|la| la.id).unwrap_or(0);
    let allocation_offset = last_allocation.map(|la| la.id).unwrap_or(0);

    let allocations = asset.known_allocations();

    let mut utxos = vec![];
    let mut allocation_vec = vec![];

    for (index, allocation) in allocations.into_iter().enumerate() {
        let utxo_id = utxo_offset + index as i32 + 1;
        let allocation_id = allocation_offset + index as i32 + 1;
        utxos.push(SqlAllocationUtxo {
            id: utxo_id,
            sql_asset_id: table_asset.id,
            txid: allocation.outpoint().txid.to_hex(),
            vout: allocation.outpoint().vout as i32,
        });
        allocation_vec.push(SqlAllocation {
            id: allocation_id,
            sql_allocation_utxo_id: utxo_id,
            node_id: allocation.node_id().to_hex(),
            assignment_index: allocation.index().clone() as i32,
            amount: allocation.revealed_amount().value as i64,
            blinding: allocation.revealed_amount().blinding.to_hex(),
        });
    }

    Ok((utxos, allocation_vec))
}

/// Read the associated AllocationUtxo and Allocation entries
/// with the given Asset entry executed over the given database
/// connection.
/// This will return a `Vec` which can be directly used as the
/// `known_allocations` field in the Asset Data structure.
pub fn read_allocations(
    asset: &SqlAsset,
    connection: &SqliteConnection,
) -> Result<Vec<Allocation>, SqlCacheError> {
    // Get the associated utxo with asset entry
    let utxo_list = SqlAllocationUtxo::belonging_to(asset)
        .load::<SqlAllocationUtxo>(connection)?;

    // Get the associated allocations with the above utxos
    let allocation_list = SqlAllocation::belonging_to(&utxo_list)
        .load::<SqlAllocation>(connection)?;

    let utxo_list = utxo_list
        .into_iter()
        .map(|utxo| (utxo.id, utxo))
        .collect::<BTreeMap<_, _>>();

    let mut allocations = Vec::new();
    for allocation in allocation_list {
        if let Some(utxo) = utxo_list.get(&allocation.sql_allocation_utxo_id) {
            allocations.push(allocation.to_allocation(utxo)?)
        } else {
            return Err(SqlCacheError::NotFound);
        }
    }

    Ok(allocations)
}
