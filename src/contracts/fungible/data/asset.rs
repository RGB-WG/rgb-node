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

use core::convert::{TryFrom, TryInto};
use core::ops::{Add, AddAssign};
use diesel::prelude::*;
use std::collections::BTreeMap;
use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::contracts::fungible::cache::models::{
    read_allocations, read_inflation, SqlAllocation, SqlAllocationUtxo,
    SqlAsset, SqlIssue,
};
use crate::contracts::fungible::cache::SqlCacheError;
use lnpbp::bitcoin;
use lnpbp::bitcoin::hashes::Hash;
use lnpbp::bitcoin::{OutPoint, Txid};
use lnpbp::bitcoin_hashes::hex::FromHex;
use lnpbp::bp;
use lnpbp::rgb::prelude::*;
use lnpbp::rgb::seal::WitnessVoutError;
use lnpbp::secp256k1zkp::key::SecretKey;
use lnpbp::secp256k1zkp::Secp256k1;

use super::schema::{self, FieldType, OwnedRightsType};
use crate::error::ServiceErrorDomain;

pub type AccountingValue = f32;

#[derive(
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Display,
    Default,
)]
#[display(Debug)]
pub struct AccountingAmount(AtomicValue, u8);

impl AccountingAmount {
    #[inline]
    pub fn transmutate(
        fractional_bits: u8,
        accounting_value: AccountingValue,
    ) -> AtomicValue {
        AccountingAmount::from_fractioned_accounting_value(
            fractional_bits,
            accounting_value,
        )
        .atomic_value()
    }

    #[inline]
    pub fn from_asset_accounting_value(
        asset: &Asset,
        accounting_value: AccountingValue,
    ) -> Self {
        let bits = asset.fractional_bits;
        let full = (accounting_value.trunc() as u64) << bits as u64;
        let fract = accounting_value.fract() as u64;
        Self(full + fract, asset.fractional_bits)
    }

    #[inline]
    pub fn from_fractioned_atomic_value(
        fractional_bits: u8,
        atomic_value: AtomicValue,
    ) -> Self {
        Self(atomic_value, fractional_bits)
    }

    #[inline]
    pub fn from_fractioned_accounting_value(
        fractional_bits: u8,
        accounting_value: AccountingValue,
    ) -> Self {
        let fract = (accounting_value.fract()
            * 10u64.pow(fractional_bits as u32) as AccountingValue)
            as u64;
        Self(accounting_value.trunc() as u64 + fract, fractional_bits)
    }

    #[inline]
    pub fn from_asset_atomic_value(
        asset: &Asset,
        atomic_value: AtomicValue,
    ) -> Self {
        Self(atomic_value, asset.fractional_bits)
    }

    #[inline]
    pub fn accounting_value(&self) -> AccountingValue {
        let full = self.0 >> self.1;
        let fract = self.0 ^ (full << self.1);
        full as AccountingValue
            + fract as AccountingValue
                / 10u64.pow(self.1 as u32) as AccountingValue
    }

    #[inline]
    pub fn atomic_value(&self) -> AtomicValue {
        self.0
    }

    #[inline]
    pub fn fractional_bits(&self) -> u8 {
        self.1
    }
}

impl Add for AccountingAmount {
    type Output = AccountingAmount;
    fn add(self, rhs: Self) -> Self::Output {
        if self.fractional_bits() != rhs.fractional_bits() {
            panic!("Addition of amounts with different fractional bits")
        } else {
            AccountingAmount::from_fractioned_atomic_value(
                self.fractional_bits(),
                self.atomic_value() + rhs.atomic_value(),
            )
        }
    }
}

impl AddAssign for AccountingAmount {
    fn add_assign(&mut self, rhs: Self) {
        if self.fractional_bits() != rhs.fractional_bits() {
            panic!("Addition of amounts with different fractional bits")
        } else {
            self.0 += rhs.0
        }
    }
}

#[derive(Clone, Getters, Serialize, Deserialize, PartialEq, Debug, Display)]
#[display(Debug)]
pub struct Asset {
    id: ContractId, // This is a unique primary key
    ticker: String,
    name: String,
    description: Option<String>,
    supply: Supply,
    #[serde(with = "serde_with::rust::display_fromstr")]
    chain: bp::Chain,
    fractional_bits: u8,
    date: NaiveDateTime,
    known_issues: Vec<Issue>,
    /// Specifies outpoints which when spent may indicate inflation happenning
    /// up to specific amount.
    known_inflation: BTreeMap<bitcoin::OutPoint, AccountingAmount>,
    /// Specifies max amount to which asset can be inflated without our
    /// knowledge
    unknown_inflation: AccountingAmount,
    /// Specifies outpoints controlling certain amounts of assets
    known_allocations: BTreeMap<bitcoin::OutPoint, Vec<Allocation>>,
}

impl Asset {
    /// Create an Asset structure from an sqlite asset table entry.
    /// This fetches all the other tables for entries associated with the given
    /// asset and recreates the full Asset structure. This should be used
    /// while reading Asset data from database table.
    pub fn from_sql_asset(
        table_value: &SqlAsset,
        connection: &SqliteConnection,
    ) -> Result<Self, SqlCacheError> {
        let (known_inflation, unknown_inflation) =
            read_inflation(table_value, connection)?;

        let known_table_issues =
            SqlIssue::belonging_to(table_value).load::<SqlIssue>(connection)?;

        let mut known_issues = vec![];

        for issue in known_table_issues {
            known_issues.push(Issue::from_sql_issue(
                issue,
                table_value.fractional_bits[0],
            )?)
        }

        Ok(Self {
            id: ContractId::from_hex(&table_value.contract_id[..])?,
            ticker: table_value.ticker.clone(),
            name: table_value.asset_name.clone(),
            description: table_value.asset_description.clone(),
            supply: Supply::from_sql_asset(&table_value),
            chain: bp::Chain::from_str(&table_value.chain[..])?,
            fractional_bits: table_value.fractional_bits[0],
            date: table_value.asset_date,
            known_issues: known_issues,
            known_inflation: known_inflation,
            unknown_inflation: unknown_inflation,
            known_allocations: read_allocations(&table_value, connection)?,
        })
    }
}

#[derive(Clone, Getters, Serialize, Deserialize, PartialEq, Debug, Display)]
#[display(Debug)]
pub struct Allocation {
    // Unique primary key is `node_id` + `index`
    node_id: NodeId,
    /// Index of the assignment of ownership right type within the node
    index: u16,
    /// Copy of the outpoint from corresponding entry in
    /// `Asset::known_allocations`
    outpoint: bitcoin::OutPoint,
    value: value::Revealed,
}

impl Allocation {
    /// Create an Allocation structure by reading the
    /// corresponding Allocation and AllocationUtxo table entries.
    pub fn from_sql_allocation(
        table_value: &SqlAllocation,
        outpoint: &SqlAllocationUtxo,
    ) -> Result<Self, SqlCacheError> {
        Ok(Self {
            node_id: NodeId::from_hex(&table_value.node_id[..])?,
            index: table_value.assignment_index as u16,
            outpoint: OutPoint {
                txid: Txid::from_hex(&outpoint.txid[..])?,
                vout: outpoint.vout as u32,
            },
            value: value::Revealed {
                value: table_value.amount as AtomicValue,
                blinding: SecretKey::from_slice(
                    &Secp256k1::new(),
                    &Vec::<u8>::from_hex(&table_value.blinding[..])?[..],
                )?,
            },
        })
    }
}

#[derive(
    Clone,
    Copy,
    Getters,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Display,
    Default,
)]
#[display(Debug)]
pub struct Supply {
    // Sum of all issued amounts
    known_circulating: AccountingAmount,
    // Specifies if all issuances are known (i.e. there are data for issue
    // state transitions for all already spent `inflation`
    // single-use-seals). In this case `known_circulating` will be equal to
    // `total_circulating`. The parameter is option since the fact that the
    // UTXO is spend may be unknown without blockchain access
    is_issued_known: Option<bool>,
    // We always know total supply, b/c even for assets without defined cap the
    // cap *de facto* equals to u64::MAX
    max_cap: AccountingAmount,
}

impl Supply {
    pub fn total_circulating(&self) -> Option<AccountingAmount> {
        if self.is_issued_known.unwrap_or(false) {
            Some(self.known_circulating)
        } else {
            None
        }
    }

    /// Supply do not have a corresponding table in the database.
    /// The concerned values are written in the Asset table itself.
    /// This reads up an asset table entry and create the corresponding
    /// supply structure.
    pub fn from_sql_asset(table_value: &SqlAsset) -> Self {
        Self {
            known_circulating:
                AccountingAmount::from_fractioned_accounting_value(
                    table_value.fractional_bits[0],
                    table_value.known_circulating_supply as AccountingValue,
                ),
            is_issued_known: table_value.is_issued_known,
            max_cap: AccountingAmount::from_fractioned_accounting_value(
                table_value.fractional_bits[0],
                table_value.max_cap as AccountingValue,
            ),
        }
    }
}

#[derive(
    Clone,
    Copy,
    Getters,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Display,
)]
#[display(Debug)]
pub struct Issue {
    // Unique primary key; equals to the state transition id that performs
    // issuance (i.e. of `issue` type)
    id: NodeId,
    // Foreign key for linking to assets
    asset_id: ContractId,
    // In db we can store it as a simple u64 field converting it on read/write
    // using `fractional_bits` parameter of the asset
    amount: AccountingAmount,
    /// Indicates transaction output which had an assigned inflation right and
    /// which spending produced this issue. `None` signifies that the issue
    /// was produced by genesis (i.e. it is a primary issue)
    origin: Option<bitcoin::OutPoint>,
}

impl Issue {
    pub fn is_primary(&self) -> bool {
        self.origin.is_none()
    }

    pub fn is_secondary(&self) -> bool {
        self.origin.is_some()
    }

    /// Create an Issue structure from reading the corresponding
    /// Issue table entry in the database.
    pub fn from_sql_issue(
        table_value: SqlIssue,
        fraction_bits: u8,
    ) -> Result<Issue, SqlCacheError> {
        Ok(Issue {
            id: NodeId::from_hex(&table_value.node_id[..])?,
            asset_id: ContractId::from_hex(&table_value.contract_id[..])?,
            amount: AccountingAmount::from_fractioned_accounting_value(
                fraction_bits,
                table_value.amount as AccountingValue,
            ),
            origin: match (table_value.origin_txid, table_value.origin_vout) {
                (Some(txid), Some(vout)) => Some(OutPoint {
                    txid: Txid::from_hex(&txid[..])?,
                    vout: vout as u32,
                }),
                _ => None,
            },
        })
    }
}

impl Asset {
    pub fn add_issue(&self, _issue: Transition) -> Supply {
        unimplemented!()
    }

    #[inline]
    pub fn allocations(
        &self,
        seal: &bitcoin::OutPoint,
    ) -> Option<&Vec<Allocation>> {
        self.known_allocations.get(seal)
    }

    pub fn add_allocation(
        &mut self,
        outpoint: bitcoin::OutPoint,
        node_id: NodeId,
        index: u16,
        value: value::Revealed,
    ) -> bool {
        let new_allocation = Allocation {
            node_id,
            index,
            outpoint,
            value,
        };
        let allocations =
            self.known_allocations.entry(outpoint).or_insert(vec![]);
        if !allocations.contains(&new_allocation) {
            allocations.push(new_allocation);
            true
        } else {
            false
        }
    }

    pub fn remove_allocation(
        &mut self,
        outpoint: bitcoin::OutPoint,
        node_id: NodeId,
        index: u16,
        value: value::Revealed,
    ) -> bool {
        let old_allocation = Allocation {
            node_id,
            index,
            outpoint,
            value,
        };
        let allocations =
            self.known_allocations.entry(outpoint).or_insert(vec![]);
        if let Some(index) =
            allocations.iter().position(|a| *a == old_allocation)
        {
            allocations.remove(index);
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// Can't read asset data: provided information does not match schema:
    /// {_0}
    #[from]
    Schema(schema::Error),

    /// Genesis defines a seal referencing witness transaction while there
    /// can't be a witness transaction for genesis
    #[from(WitnessVoutError)]
    GenesisSeal,
}

impl From<Error> for ServiceErrorDomain {
    fn from(err: Error) -> Self {
        ServiceErrorDomain::Schema(format!("{}", err))
    }
}

impl TryFrom<Genesis> for Asset {
    type Error = Error;

    fn try_from(genesis: Genesis) -> Result<Self, Self::Error> {
        if genesis.schema_id() != schema::schema().schema_id() {
            Err(schema::Error::WrongSchemaId)?;
        }
        let genesis_meta = genesis.metadata();
        let fractional_bits = *genesis_meta
            .u8(*FieldType::Precision)
            .first()
            .ok_or(schema::Error::NotAllFieldsPresent)?;
        let supply = AccountingAmount::from_fractioned_atomic_value(
            fractional_bits,
            *genesis_meta
                .u64(*FieldType::IssuedSupply)
                .first()
                .ok_or(schema::Error::NotAllFieldsPresent)?,
        );
        let mut known_inflation = BTreeMap::<_, _>::default();
        let mut unknown_inflation = AccountingAmount::default();

        for assignment in
            genesis.owned_rights_by_type(*OwnedRightsType::Inflation)
        {
            for state in assignment.to_custom_state() {
                match state {
                    OwnedState::Revealed {
                        seal_definition,
                        assigned_state,
                    } => {
                        known_inflation.insert(
                            seal_definition.try_into()?,
                            AccountingAmount::from_fractioned_atomic_value(
                                fractional_bits,
                                assigned_state.u64().ok_or(
                                    schema::Error::NotAllFieldsPresent,
                                )?,
                            ),
                        );
                    }
                    OwnedState::ConfidentialSeal { assigned_state, .. } => {
                        if unknown_inflation.atomic_value() < core::u64::MAX {
                            unknown_inflation +=
                                AccountingAmount::from_fractioned_atomic_value(
                                    fractional_bits,
                                    assigned_state.u64().ok_or(
                                        schema::Error::NotAllFieldsPresent,
                                    )?,
                                )
                        };
                    }
                    _ => {
                        unknown_inflation =
                            AccountingAmount::from_fractioned_atomic_value(
                                fractional_bits,
                                core::u64::MAX,
                            );
                    }
                }
            }
        }

        let node_id = NodeId::from_inner(genesis.contract_id().into_inner());
        let issue = Issue {
            id: genesis.node_id(),
            asset_id: genesis.contract_id(),
            amount: supply.clone(),
            origin: None, // This is a primary issue, so no origin here
        };
        let mut known_allocations =
            BTreeMap::<bitcoin::OutPoint, Vec<Allocation>>::default();
        for assignment in genesis.owned_rights_by_type(*OwnedRightsType::Assets)
        {
            assignment
                .to_discrete_state()
                .into_iter()
                .enumerate()
                .for_each(|(index, assign)| {
                    if let OwnedState::Revealed {
                        seal_definition:
                            seal::Revealed::TxOutpoint(outpoint_reveal),
                        assigned_state,
                    } = assign
                    {
                        known_allocations
                            .entry(outpoint_reveal.clone().into())
                            .or_insert(vec![])
                            .push(Allocation {
                                node_id,
                                index: index as u16,
                                outpoint: outpoint_reveal.into(),
                                value: assigned_state,
                            })
                    }
                });
        }
        Ok(Self {
            id: genesis.contract_id(),
            chain: genesis.chain().clone(),
            ticker: genesis_meta
                .string(*FieldType::Ticker)
                .first()
                .ok_or(schema::Error::NotAllFieldsPresent)?
                .clone(),
            name: genesis_meta
                .string(*FieldType::Name)
                .first()
                .ok_or(schema::Error::NotAllFieldsPresent)?
                .clone(),
            description: genesis_meta
                .string(*FieldType::ContractText)
                .first()
                .cloned(),
            supply: Supply {
                known_circulating: supply,
                is_issued_known: None,
                max_cap: genesis
                    .owned_rights_by_type(*OwnedRightsType::Inflation)
                    .map(|assignments| {
                        AccountingAmount::from_fractioned_atomic_value(
                            fractional_bits,
                            assignments
                                .known_state_data()
                                .into_iter()
                                .map(|data| match data {
                                    data::Revealed::U64(cap) => *cap,
                                    _ => 0,
                                })
                                .sum(),
                        )
                    })
                    .unwrap_or(supply),
            },
            fractional_bits,
            date: NaiveDateTime::from_timestamp(
                *genesis_meta
                    .i64(*FieldType::Timestamp)
                    .first()
                    .ok_or(schema::Error::NotAllFieldsPresent)?,
                0,
            ),
            known_inflation,
            unknown_inflation,
            known_issues: vec![issue],
            // we assume that each genesis allocation with revealed amount
            // and known seal (they are always revealed together) belongs to us
            known_allocations,
        })
    }
}
