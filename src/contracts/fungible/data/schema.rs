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
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
use std::collections::BTreeSet;

use lnpbp::features;
use lnpbp::rgb::schema::{
    script, AssignmentAction, Bits, DataFormat, DiscreteFiniteFieldFormat, GenesisSchema,
    Occurences, Schema, StateFormat, StateSchema, TransitionSchema,
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

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, ToPrimitive, FromPrimitive,
)]
#[display(Debug)]
#[repr(u16)]
pub enum FieldType {
    Ticker = 0,
    Name = 1,
    Description = 2,
    TotalSupply = 3,
    IssuedSupply = 4,
    DustLimit = 5,
    Precision = 6,
    Timestamp = 7,
    HistoryProof = 8,
    HistoryProofFormat = 9,
}

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, ToPrimitive, FromPrimitive,
)]
#[display(Debug)]
#[repr(u16)]
pub enum OwnedRightsType {
    Issue = 0,
    Assets = 1,
    Epoch = 2,
    Replacement = 3,
    Renomination = 4,
}

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, ToPrimitive, FromPrimitive,
)]
#[display(Debug)]
#[repr(u16)]
pub enum TransitionType {
    Issue = 0,
    Transfer = 1,
    Epoch = 2,
    Replacement = 3,
    Renomination = 4,
}

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, ToPrimitive, FromPrimitive,
)]
#[display(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum HistoryProofFormat {
    ProofAbsent = 0x0,
    ProovV1 = 0x1,
    ProovV2 = 0x2,
    ProovV3 = 0x3,
    ProovV4 = 0x4,
    ProovV5 = 0x5,
    ProovV6 = 0x6,
    ProovV7 = 0x7,
    ProovV8 = 0x8,
    ProovV9 = 0x9,
    ProovV10 = 0xA,
    ProovV11 = 0xB,
    ProovV12 = 0xC,
    ProovV13 = 0xD,
    ProovV14 = 0xE,
    ProovV15 = 0xF,
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
            // Ricardian contract. We use all available size, in case the
            // contract is long. If the contract still doesn't fit, a hash or
            // URL should be used instead, pointing to the full contract text
            FieldType::Description => DataFormat::String(core::u16::MAX),
            FieldType::TotalSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::Precision => DataFormat::Unsigned(Bits::Bit8, 0, 18u128),
            FieldType::IssuedSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            // While UNIX timestamps allow negative numbers; in context of RGB Schema, assets
            // can't be issued in the past before RGB or Bitcoin even existed; so we prohibit
            // all the dates before RGB release
            // TODO: Update lower allowed timestamp value with the first RGB release
            //       Current lower time limit is 07/04/2020 @ 1:54pm (UTC)
            FieldType::Timestamp => DataFormat::Integer(Bits::Bit64, 1593870844, core::i64::MAX as i128),
            FieldType::HistoryProof => DataFormat::Bytes(core::u16::MAX),
            FieldType::HistoryProofFormat => DataFormat::Enum(HistoryProofFormat::all())
        },
        owned_right_types: type_map! {
            OwnedRightsType::Issue => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {
                    AssignmentAction::Validate => script::Procedure::Standard(script::StandardProcedure::IssueControl)
                }
            },
            OwnedRightsType::Assets => StateSchema {
                format: StateFormat::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit),
                abi: bmap! {
                    AssignmentAction::Validate => script::Procedure::Standard(script::StandardProcedure::ConfidentialAmount)
                }
            },
            OwnedRightsType::Epoch => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {
                    AssignmentAction::Validate => script::Procedure::NoOp
                }
            },
            OwnedRightsType::Replacement => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {
                    AssignmentAction::Validate => script::Procedure::Standard(script::StandardProcedure::Replacement)
                }
            },
            OwnedRightsType::Renomination => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {
                    AssignmentAction::Validate => script::Procedure::NoOp
                }
            }
        },
        public_right_types: Default::default(),
        genesis: GenesisSchema {
            metadata: type_map! {
                FieldType::Ticker => Occurences::Once,
                FieldType::Name => Occurences::Once,
                FieldType::Description => Occurences::NoneOrOnce,
                FieldType::TotalSupply => Occurences::Once,
                FieldType::IssuedSupply => Occurences::Once,
                FieldType::DustLimit => Occurences::NoneOrOnce,
                FieldType::Precision => Occurences::Once,
                FieldType::Timestamp => Occurences::Once
            },
            owned_rights: type_map! {
                OwnedRightsType::Issue => Occurences::NoneOrOnce,
                OwnedRightsType::Epoch => Occurences::NoneOrOnce,
                OwnedRightsType::Assets => Occurences::NoneOrUpTo(None),
                OwnedRightsType::Renomination => Occurences::NoneOrOnce
            },
            public_rights: Default::default(),
            abi: bmap! {},
        },
        extensions: Default::default(),
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
                public_rights: Default::default(),
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
                public_rights: Default::default(),
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
                public_rights: Default::default(),
                abi: bmap! {}
            },
            TransitionType::Replacement => TransitionSchema {
                metadata: type_map! {
                    FieldType::IssuedSupply => Occurences::Once,
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
                public_rights: Default::default(),
                abi: bmap! {}
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
                public_rights: Default::default(),
                abi: bmap! {}
            }
        },
    }
}

impl Deref for FieldType {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.to_usize().expect("Any enum always fits into usize")
    }
}

impl Deref for OwnedRightsType {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.to_usize().expect("Any enum always fits into usize")
    }
}

impl Deref for TransitionType {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.to_usize().expect("Any enum always fits into usize")
    }
}

impl Deref for HistoryProofFormat {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.to_u8().expect("History proofs always fit into u8")
    }
}
