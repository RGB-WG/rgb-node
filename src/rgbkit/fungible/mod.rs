mod schema;

use super::*;
use chrono::NaiveDateTime;
use core::convert::TryFrom;
use core::option::NoneError;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

use lnpbp::bitcoin;
use lnpbp::bitcoin::secp256k1;
use lnpbp::bp;
use lnpbp::miniscript::Miniscript;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Display, Default)]
#[display_from(Display)]
pub struct Amount(rgb::Amount, u8);

impl Amount {
    #[inline]
    pub fn with_asset_coins(asset: &Asset, coins: f32) -> Self {
        let bits = asset.fractional_bits;
        let full = (coins.trunc() as u64) << bits as u64;
        let fract = coins.fract() as u64;
        Self(full + fract, asset.fractional_bits)
    }

    #[inline]
    fn with_sats_precision(sats: u64, fractional_bits: u8) -> Self {
        Self(sats, fractional_bits)
    }

    #[inline]
    fn with_asset_sats(asset: &Asset, sats: u64) -> Self {
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
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display)]
#[display_from(Display)]
pub struct Asset {
    id: ContractId,
    ticker: String,
    name: String,
    description: Option<String>,
    supply: Supply,
    dust_limit: Amount,
    network: bp::MagicNumber,
    fractional_bits: u8,
    unspent_issue_txo: Option<bitcoin::OutPoint>,
    known_issues: Vec<LinkedList<Issue>>,
    known_allocations: Vec<Allocation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub struct Issue {
    pub id: ContractId,
    /// A point that has to be monitored to detect next issuance
    pub txo: Option<bitcoin::OutPoint>,
    pub supply: Amount,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub struct Allocation {
    pub amount: Amount,
    pub seal: bitcoin::OutPoint,
    pub payment: Option<Payment>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, Error, From)]
#[display_from(Display)]
pub enum Error {
    #[derive_from(NoneError)]
    IncorrectSchema,
}

impl TryFrom<Genesis> for Asset {
    type Error = Error;

    fn try_from(genesis: Genesis) -> Result<Self, Self::Error> {
        if genesis.schema.clone().schema_id() != schema::schema().schema_id() {
            Err(Error::IncorrectSchema)?;
        }
        let fractional_bits = genesis
            .u8(schema::FieldType::FractionalBits.to_usize()?)?
            .first()?
            .clone();
        let supply = Amount::with_sats_precision(
            genesis
                .u64(schema::FieldType::IssuedSupply.to_usize()?)?
                .first()?
                .clone(),
            fractional_bits,
        );
        Ok(Self {
            id: genesis.clone().contract_id(),
            network: genesis.clone().network.as_magic(),
            ticker: genesis
                .string(schema::FieldType::Ticker.to_usize()?)?
                .first()?
                .clone(),
            name: genesis
                .string(schema::FieldType::Name.to_usize()?)?
                .first()?
                .clone(),
            description: genesis
                .string(schema::FieldType::Description.to_usize()?)?
                .first()
                .cloned(),
            supply: Supply {
                known_circulating: supply.clone(),
                total: Some(Amount::with_sats_precision(
                    genesis
                        .u64(schema::FieldType::TotalSupply.to_usize()?)?
                        .first()?
                        .clone(),
                    fractional_bits,
                )),
            },
            dust_limit: Amount::with_sats_precision(
                genesis
                    .u64(schema::FieldType::DustLimit.to_usize()?)?
                    .first()?
                    .clone(),
                fractional_bits,
            ),
            fractional_bits,
            unspent_issue_txo: None,
            known_issues: vec![list! { Issue {
                id: genesis.clone().contract_id(),
                txo: genesis.defined_seals(schema::AssignmentsType::Issue.to_usize()?)?
                    .first()
                    .and_then(|i| bitcoin::OutPoint::try_from(i.clone()).ok()),
                supply
            } }],
            // we assume that each genesis allocation with revealed amount
            // and known seal (they are always revealed together) belongs to us
            known_allocations: genesis
                .assignments()
                .get(&schema::AssignmentsType::Assets.to_usize()?)
                .iter()
                .filter_map(|variant| match variant {
                    AssignmentsVariant::Homomorphic(tree) => Some(
                        tree.iter()
                            .filter_map(|assign| match assign {
                                rgb::Assignment::Revealed {
                                    seal_definition: rgb::seal::Revealed::TxOutpoint(outpoint),
                                    assigned_state,
                                } => Some(Allocation {
                                    amount: Amount::with_sats_precision(
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

impl Asset {
    pub fn network(&self) -> bp::Network {
        bp::Network::from_magic(self.network)
    }

    pub fn add_issue(&self, issue: Transition) -> Result<Supply, Error> {
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
    pub fn dust_limit(&self) -> Amount {
        self.dust_limit.clone()
    }

    #[inline]
    pub fn fractional_bits(&self) -> u8 {
        self.fractional_bits
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Display, Default)]
#[display_from(Display)]
pub struct Supply {
    pub known_circulating: Amount,
    pub total: Option<Amount>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Display)]
#[display_from(Debug)]
pub struct Payment {
    pub date: NaiveDateTime,
    pub payer: Payer,
}

impl Payment {}
