use chrono::NaiveDateTime;
use core::convert::TryFrom;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

use bitcoin::secp256k1;
use lnpbp::bitcoin;

use lnpbp::bp;
use lnpbp::miniscript::Miniscript;
use lnpbp::rgb::prelude::*;

use super::schema::{AssignmentsType, FieldType};
use super::{schema, SchemaError};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Display, Default)]
#[display_from(Display)]
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

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Display)]
#[display_from(Display)]
pub struct Asset {
    id: ContractId,
    ticker: String,
    name: String,
    description: Option<String>,
    supply: Supply,
    dust_limit: Coins,
    network: bp::MagicNumber,
    fractional_bits: u8,
    date: NaiveDateTime,
    unspent_issue_txo: Option<bitcoin::OutPoint>,
    known_issues: Vec<LinkedList<Issue>>,
    known_allocations: Vec<Allocation>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Display, Default)]
#[display_from(Display)]
pub struct Supply {
    pub known_circulating: Coins,
    pub total: Option<Coins>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub struct Issue {
    pub id: ContractId,
    /// A point that has to be monitored to detect next issuance
    pub txo: Option<bitcoin::OutPoint>,
    pub supply: Coins,
}

/*
wrapper!(
    Allocations,
    Vec<Allocation>,
    doc = "Allocation of coins to seal definitions",
    derive = [PartialEq]
);
*/

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Display)]
#[display_from(Debug)]
pub struct Allocation {
    pub value: Coins,
    pub seal: bitcoin::OutPoint,
    pub payment: Option<Payment>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub struct Payment {
    pub date: NaiveDateTime,
    pub payer: Payer,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Display)]
#[display_from(Debug)]
#[non_exhaustive]
pub enum Payer {
    Genesis(ContractId),
    BitcoinPubkey(bitcoin::PublicKey),
    BitcoinMultisig(Vec<bitcoin::PublicKey>, u8),
    BitcoinScript(Miniscript<bitcoin::PublicKey>),
    Tapscript(Miniscript<bitcoin::PublicKey>),
    LightningNode(secp256k1::PublicKey),
}

impl Asset {
    pub fn network(&self) -> bp::Network {
        bp::Network::from_magic(self.network)
    }

    pub fn add_issue(&self, issue: Transition) -> Supply {
        unimplemented!()
    }

    #[inline]
    pub fn ticker(&self) -> &str {
        self.ticker.as_str()
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        match &self.description {
            None => None,
            Some(s) => Some(s.as_str()),
        }
    }

    #[inline]
    pub fn supply(&self) -> Supply {
        self.supply.clone()
    }

    #[inline]
    pub fn dust_limit(&self) -> Coins {
        self.dust_limit.clone()
    }

    #[inline]
    pub fn fractional_bits(&self) -> u8 {
        self.fractional_bits
    }

    #[inline]
    pub fn date(&self) -> NaiveDateTime {
        self.date
    }
}

impl TryFrom<Genesis> for Asset {
    type Error = SchemaError;

    fn try_from(genesis: Genesis) -> Result<Self, Self::Error> {
        if genesis.schema_id() != schema::schema().schema_id() {
            Err(SchemaError::NotAllFieldsPresent)?;
        }
        let fractional_bits = genesis.u8(-FieldType::FractionalBits)?;
        let supply =
            Coins::with_sats_precision(genesis.u64(-FieldType::IssuedSupply)?, fractional_bits);
        Ok(Self {
            id: genesis.contract_id(),
            network: genesis.network().as_magic(),
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
            date: NaiveDateTime::from_timestamp(genesis.u32(-FieldType::Timestamp)? as i64, 0),
            unspent_issue_txo: None,
            known_issues: vec![list! { Issue {
                id: genesis.contract_id(),
                txo: genesis.defined_seals(-AssignmentsType::Issue)?
                    .first()
                    .and_then(|i| bitcoin::OutPoint::try_from(i.clone()).ok()),
                supply
            } }],
            // we assume that each genesis allocation with revealed amount
            // and known seal (they are always revealed together) belongs to us
            known_allocations: genesis
                .assignments()
                .get(&-AssignmentsType::Assets)
                .iter()
                .filter_map(|variant| match variant {
                    AssignmentsVariant::Homomorphic(tree) => Some(
                        tree.iter()
                            .filter_map(|assign| match assign {
                                Assignment::Revealed {
                                    seal_definition: seal::Revealed::TxOutpoint(outpoint),
                                    assigned_state,
                                } => Some(Allocation {
                                    value: Coins::with_sats_precision(
                                        assigned_state.amount,
                                        fractional_bits,
                                    ),
                                    seal: outpoint.clone().into(),
                                    payment: None,
                                }),
                                _ => None,
                            })
                            .collect::<Vec<Allocation>>(),
                    ),
                    _ => None,
                })
                .flatten()
                .collect(),
        })
    }
}
