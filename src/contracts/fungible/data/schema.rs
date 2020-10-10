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

use core::ops::Deref;
use std::collections::BTreeSet;

use lnpbp::features;
use lnpbp::rgb::schema::{
    script, AssignmentAction, Bits, DataFormat, DiscreteFiniteFieldFormat, GenesisAction,
    GenesisSchema, Occurences, Schema, StateFormat, StateSchema, TransitionAction,
    TransitionSchema,
};

use crate::error::ServiceErrorDomain;
use crate::type_map;

#[derive(Debug, Display, Error, From)]
#[display(Debug)]
pub enum SchemaError {
    #[from(core::option::NoneError)]
    NotAllFieldsPresent,

    WrongSchemaId,
}

impl From<SchemaError> for ServiceErrorDomain {
    fn from(err: SchemaError) -> Self {
        ServiceErrorDomain::Schema(format!("{}", err))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
pub enum FieldType {
    Ticker,
    Name,
    Description,
    Precision,
    TotalSupply,
    IssuedSupply,
    ReplacedSupply,
    BurnedSupply,
    Timestamp,
    BurnedUtxo,
    HistoryProof,
    HistoryProofFormat,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
#[repr(u16)]
pub enum OwnedRightsType {
    Issue,
    Assets,
    Epoch,
    Replacement,
    Renomination,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
#[repr(u16)]
pub enum TransitionType {
    Issue,
    Transfer,
    Epoch,
    Replacement,
    Renomination,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum HistoryProofFormat {
    ProofAbsent,
    ProovV1,
    ProovV2,
    ProovV3,
    ProovV4,
    ProovV5,
    ProovV6,
    ProovV7,
    ProovV8,
    ProovV9,
    ProovV10,
    ProovV11,
    ProovV12,
    ProovV13,
    ProovV14,
    ProovV15,
}

impl HistoryProofFormat {
    pub fn all() -> BTreeSet<u8> {
        bset![
            *HistoryProofFormat::ProofAbsent,
            *HistoryProofFormat::ProovV1,
            *HistoryProofFormat::ProovV2,
            *HistoryProofFormat::ProovV3,
            *HistoryProofFormat::ProovV4,
            *HistoryProofFormat::ProovV5,
            *HistoryProofFormat::ProovV6,
            *HistoryProofFormat::ProovV7,
            *HistoryProofFormat::ProovV8,
            *HistoryProofFormat::ProovV9,
            *HistoryProofFormat::ProovV10,
            *HistoryProofFormat::ProovV11,
            *HistoryProofFormat::ProovV12,
            *HistoryProofFormat::ProovV13,
            *HistoryProofFormat::ProovV14,
            *HistoryProofFormat::ProovV15
        ]
    }
}

pub fn schema() -> Schema {
    Schema {
        rgb_features: features::FlagVec::new(),
        root_id: Default::default(),
        field_types: type_map! {
            // Rational: if we will use just 26 letters of English alphabet (and
            // we are not limited by them), we will have 26^8 possible tickers,
            // i.e. > 208 trillions, which is sufficient amount
            FieldType::Ticker => DataFormat::String(8),
            FieldType::Name => DataFormat::String(256),
            // Description may contain URL, text or text representation of
            // Ricardian contract, up to 64kb. If the contract doesn't fit, a
            // double SHA256 hash and URL should be used instead, pointing to
            // the full contract text, where hash must be represented by a
            // hexadecimal string, optionally followed by `\n` and text URL
            FieldType::Description => DataFormat::String(core::u16::MAX),
            FieldType::TotalSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::Precision => DataFormat::Unsigned(Bits::Bit8, 0, 18u128),
            FieldType::IssuedSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::ReplacedSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::BurnedSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            // While UNIX timestamps allow negative numbers; in context of RGB
            // Schema, assets can't be issued in the past before RGB or Bitcoin
            // even existed; so we prohibit all the dates before RGB release
            // TODO: Update lower allowed timestamp value with the first RGB release
            //       Current lower time limit is 07/04/2020 @ 1:54pm (UTC)
            FieldType::Timestamp => DataFormat::Integer(Bits::Bit64, 1593870844, core::i64::MAX as i128),
            FieldType::HistoryProof => DataFormat::Bytes(core::u16::MAX),
            FieldType::HistoryProofFormat => DataFormat::Enum(HistoryProofFormat::all()),
            FieldType::BurnedUtxo => DataFormat::TxOutPoint
        },
        owned_right_types: type_map! {
            OwnedRightsType::Issue => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {}
            },
            OwnedRightsType::Assets => StateSchema {
                format: StateFormat::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit),
                abi: bmap! {
                    AssignmentAction::Validate => script::Procedure::Embedded(script::StandardProcedure::ConfidentialAmount)
                }
            },
            OwnedRightsType::Epoch => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {}
            },
            OwnedRightsType::Replacement => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {}
            },
            OwnedRightsType::Renomination => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {}
            }
        },
        public_right_types: bset![],
        genesis: GenesisSchema {
            metadata: type_map! {
                FieldType::Ticker => Occurences::Once,
                FieldType::Name => Occurences::Once,
                FieldType::Description => Occurences::NoneOrOnce,
                FieldType::TotalSupply => Occurences::Once,
                FieldType::IssuedSupply => Occurences::Once,
                FieldType::Precision => Occurences::Once,
                FieldType::Timestamp => Occurences::Once
            },
            owned_rights: type_map! {
                OwnedRightsType::Issue => Occurences::NoneOrOnce,
                OwnedRightsType::Epoch => Occurences::NoneOrOnce,
                OwnedRightsType::Assets => Occurences::NoneOrUpTo(None),
                OwnedRightsType::Renomination => Occurences::NoneOrOnce
            },
            public_rights: bset![],
            abi: bmap! {
                GenesisAction::Validate => script::Procedure::Embedded(script::StandardProcedure::IssueControl)
            },
        },
        extensions: bmap![],
        transitions: type_map! {
            TransitionType::Issue => TransitionSchema {
                metadata: type_map! {
                    FieldType::IssuedSupply => Occurences::Once
                },
                closes: type_map! {
                    OwnedRightsType::Issue => Occurences::Once
                },
                owned_rights: type_map! {
                    OwnedRightsType::Issue => Occurences::NoneOrOnce,
                    OwnedRightsType::Epoch => Occurences::NoneOrOnce,
                    OwnedRightsType::Assets => Occurences::NoneOrUpTo(None)
                },
                public_rights: bset! [],
                abi: bmap! {}
            },
            TransitionType::Transfer => TransitionSchema {
                metadata: type_map! {},
                closes: type_map! {
                    OwnedRightsType::Assets => Occurences::OnceOrUpTo(None)
                },
                owned_rights: type_map! {
                    OwnedRightsType::Assets => Occurences::NoneOrUpTo(None)
                },
                public_rights: bset! [],
                abi: bmap! {}
            },
            TransitionType::Epoch => TransitionSchema {
                metadata: type_map! {},
                closes: type_map! {
                    OwnedRightsType::Epoch => Occurences::Once
                },
                owned_rights: type_map! {
                    OwnedRightsType::Epoch => Occurences::NoneOrOnce,
                    OwnedRightsType::Replacement => Occurences::NoneOrOnce
                },
                public_rights: bset! [],
                abi: bmap! {}
            },
            TransitionType::Replacement => TransitionSchema {
                metadata: type_map! {
                    FieldType::IssuedSupply => Occurences::Once,
                    FieldType::BurnedUtxo => Occurences::OnceOrUpTo(None),
                    FieldType::HistoryProofFormat => Occurences::Once,
                    FieldType::HistoryProof => Occurences::NoneOrUpTo(None)
                },
                closes: type_map! {
                    OwnedRightsType::Replacement => Occurences::Once
                },
                owned_rights: type_map! {
                    OwnedRightsType::Replacement => Occurences::NoneOrOnce,
                    OwnedRightsType::Assets => Occurences::OnceOrUpTo(None)
                },
                public_rights: bset! [],
                abi: bmap! {
                    TransitionAction::Validate => script::Procedure::Embedded(script::StandardProcedure::ProofOfBurn)
                }
            },
            TransitionType::Renomination => TransitionSchema {
                metadata: type_map! {
                    FieldType::Ticker => Occurences::NoneOrOnce,
                    FieldType::Name => Occurences::NoneOrOnce,
                    FieldType::Description => Occurences::NoneOrOnce,
                    FieldType::Precision => Occurences::NoneOrOnce
                },
                closes: type_map! {
                    OwnedRightsType::Renomination => Occurences::Once
                },
                owned_rights: type_map! {
                    OwnedRightsType::Renomination => Occurences::NoneOrOnce
                },
                public_rights: bset! [],
                abi: bmap! {}
            }
        },
    }
}

impl Deref for FieldType {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        match self {
            FieldType::Ticker => &0,
            FieldType::Name => &1,
            FieldType::Description => &2,
            FieldType::Precision => &3,
            FieldType::TotalSupply => &4,
            FieldType::IssuedSupply => &5,
            FieldType::ReplacedSupply => &6,
            FieldType::BurnedSupply => &7,
            FieldType::BurnedUtxo => &8,
            FieldType::Timestamp => &9,
            FieldType::HistoryProof => &10,
            FieldType::HistoryProofFormat => &11,
        }
    }
}

impl Deref for OwnedRightsType {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        match self {
            OwnedRightsType::Issue => &0,
            OwnedRightsType::Assets => &1,
            OwnedRightsType::Epoch => &2,
            OwnedRightsType::Replacement => &3,
            OwnedRightsType::Renomination => &4,
        }
    }
}

impl Deref for TransitionType {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        match self {
            TransitionType::Issue => &0,
            TransitionType::Transfer => &1,
            TransitionType::Epoch => &2,
            TransitionType::Replacement => &3,
            TransitionType::Renomination => &4,
        }
    }
}

impl Deref for HistoryProofFormat {
    type Target = u8;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            HistoryProofFormat::ProofAbsent => &0x0,
            HistoryProofFormat::ProovV1 => &0x1,
            HistoryProofFormat::ProovV2 => &0x2,
            HistoryProofFormat::ProovV3 => &0x3,
            HistoryProofFormat::ProovV4 => &0x4,
            HistoryProofFormat::ProovV5 => &0x5,
            HistoryProofFormat::ProovV6 => &0x6,
            HistoryProofFormat::ProovV7 => &0x7,
            HistoryProofFormat::ProovV8 => &0x8,
            HistoryProofFormat::ProovV9 => &0x9,
            HistoryProofFormat::ProovV10 => &0xA,
            HistoryProofFormat::ProovV11 => &0xB,
            HistoryProofFormat::ProovV12 => &0xC,
            HistoryProofFormat::ProovV13 => &0xD,
            HistoryProofFormat::ProovV14 => &0xE,
            HistoryProofFormat::ProovV15 => &0xF,
        }
    }
}
