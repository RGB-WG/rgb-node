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

use core::convert::TryFrom;
use std::collections::{BTreeMap, LinkedList};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use lnpbp::bitcoin;
use lnpbp::bitcoin::hashes::Hash;
use lnpbp::bp;
use lnpbp::rgb::prelude::*;

use super::schema::{AssignmentsType, FieldType};
use super::{schema, SchemaError};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Display, Default)]
#[display_from(Debug)]
pub struct Coins(Amount, u8);

impl Coins {
    #[inline]
    pub fn transmutate(coins: f32, precision: u8) -> Amount {
        Coins::with_value_precision(coins, precision).sats()
    }

    #[inline]
    pub fn with_asset_coins(asset: &Asset, coins: f32) -> Self {
        let bits = asset.fractional_bits;
        let full = (coins.trunc() as u64) << bits as u64;
        let fract = coins.fract() as u64;
        Self(full + fract, asset.fractional_bits)
    }

    #[inline]
    fn with_sats_precision(sats: Amount, fractional_bits: u8) -> Self {
        Self(sats, fractional_bits)
    }

    #[inline]
    pub(crate) fn with_value_precision(value: f32, fractional_bits: u8) -> Self {
        let fract = (value.fract() * 10u64.pow(fractional_bits as u32) as f32) as u64;
        Self(value.trunc() as u64 + fract, fractional_bits)
    }

    #[inline]
    fn with_asset_sats(asset: &Asset, sats: Amount) -> Self {
        Self(sats, asset.fractional_bits)
    }

    #[inline]
    pub fn coins(&self) -> f32 {
        let full = self.0 >> self.1;
        let fract = self.0 ^ (full << self.1);
        full as f32 + fract as f32 / 10u64.pow(self.1 as u32) as f32
    }

    #[inline]
    pub fn sats(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn fractional_bits(&self) -> u8 {
        self.1
    }
}

#[derive(Clone, Getters, Serialize, Deserialize, PartialEq, Debug, Display)]
#[display_from(Debug)]
pub struct Asset {
    id: ContractId,
    ticker: String,
    name: String,
    description: Option<String>,
    supply: Supply,
    dust_limit: Coins,
    network_magic: bp::MagicNumber,
    fractional_bits: u8,
    date: NaiveDateTime,
    unspent_issue_txo: Option<bitcoin::OutPoint>,
    known_issues: Vec<LinkedList<Issue>>,
    known_allocations: BTreeMap<bitcoin::OutPoint, Vec<Allocation>>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Display)]
#[display_from(Debug)]
pub struct Allocation {
    pub node_id: NodeId,
    // Index of the assignment within the node
    pub index: u16,
    pub amount: amount::Revealed,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Display, Default)]
#[display_from(Debug)]
pub struct Supply {
    pub known_circulating: Coins,
    pub total: Option<Coins>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Display)]
#[display_from(Debug)]
pub struct Issue {
    pub id: ContractId,
    /// A point that has to be monitored to detect next issuance
    pub txo: Option<bitcoin::OutPoint>,
    pub supply: Coins,
}

impl Asset {
    #[inline]
    pub fn network(&self) -> bp::Network {
        bp::Network::from_magic(self.network_magic)
    }

    pub fn add_issue(&self, _issue: Transition) -> Supply {
        unimplemented!()
    }

    #[inline]
    pub fn allocations(&self, seal: &bitcoin::OutPoint) -> Option<&Vec<Allocation>> {
        self.known_allocations.get(seal)
    }

    pub fn add_allocation(
        &mut self,
        seal: bitcoin::OutPoint,
        node_id: NodeId,
        index: u16,
        amount: amount::Revealed,
    ) {
        self.known_allocations
            .entry(seal)
            .or_insert(vec![])
            .push(Allocation {
                node_id,
                index,
                amount,
            });
    }
}

impl TryFrom<Genesis> for Asset {
    type Error = SchemaError;

    fn try_from(genesis: Genesis) -> Result<Self, Self::Error> {
        if genesis.schema_id() != schema::schema().schema_id() {
            Err(SchemaError::WrongSchemaId)?;
        }
        let fractional_bits = genesis.u8(-FieldType::Precision)?;
        let supply =
            Coins::with_sats_precision(genesis.u64(-FieldType::IssuedSupply)?, fractional_bits);

        let node_id = NodeId::from_inner(genesis.contract_id().into_inner());
        let issue = Issue {
            id: genesis.contract_id(),
            txo: genesis
                .known_seal_definitions_by_type(-AssignmentsType::Issue)
                .first()
                .and_then(|i| bitcoin::OutPoint::try_from((*i).clone()).ok()),
            supply: supply.clone(),
        };
        let mut known_allocations = BTreeMap::<bitcoin::OutPoint, Vec<Allocation>>::default();
        for variant in genesis.assignments_by_type(-AssignmentsType::Assets) {
            if let AssignmentsVariant::DiscreteFiniteField(tree) = variant {
                tree.iter().enumerate().for_each(|(index, assign)| {
                    if let Assignment::Revealed {
                        seal_definition: seal::Revealed::TxOutpoint(outpoint_reveal),
                        assigned_state,
                    } = assign
                    {
                        known_allocations
                            .entry(outpoint_reveal.clone().into())
                            .or_insert(vec![])
                            .push(Allocation {
                                node_id,
                                index: index as u16,
                                amount: assigned_state.clone(),
                            })
                    }
                });
            }
        }
        Ok(Self {
            id: genesis.contract_id(),
            network_magic: genesis.network().as_magic(),
            ticker: genesis.string(-FieldType::Ticker)?,
            name: genesis.string(-FieldType::Name)?,
            description: genesis.string(-FieldType::Description).next(),
            supply: Supply {
                known_circulating: supply.clone(),
                total: Some(Coins::with_sats_precision(
                    genesis.u64(-FieldType::TotalSupply)?,
                    fractional_bits,
                )),
            },
            dust_limit: Coins::with_sats_precision(
                genesis.u64(-FieldType::DustLimit)?,
                fractional_bits,
            ),
            fractional_bits,
            date: NaiveDateTime::from_timestamp(genesis.i64(-FieldType::Timestamp)?, 0),
            unspent_issue_txo: None,
            known_issues: vec![list! { issue }],
            // we assume that each genesis allocation with revealed amount
            // and known seal (they are always revealed together) belongs to us
            known_allocations,
        })
    }
}
