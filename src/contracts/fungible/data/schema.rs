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

use core::ops::Neg;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;

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

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
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
    PruneProof = 7,
    Timestamp = 8,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display(Debug)]
#[repr(u16)]
pub enum OwnedRightsType {
    Issue = 0,
    Assets = 1,
    Prune = 2,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display(Debug)]
#[repr(u16)]
pub enum TransitionType {
    Issue = 0,
    Transfer = 1,
    Prune = 2,
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
            FieldType::DustLimit => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::PruneProof => DataFormat::Bytes(core::u16::MAX),
            // While UNIX timestamps allow negative numbers; in context of RGB Schema, assets
            // can't be issued in the past before RGB or Bitcoin even existed; so we prohibit
            // all the dates before RGB release
            // TODO: Update lower allowed timestamp value with the first RGB release
            //       Current lower time limit is 07/04/2020 @ 1:54pm (UTC)
            FieldType::Timestamp => DataFormat::Integer(Bits::Bit64, 1593870844, core::i64::MAX as i128)
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
            OwnedRightsType::Prune => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {
                    AssignmentAction::Validate => script::Procedure::Standard(script::StandardProcedure::Prunning)
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
                OwnedRightsType::Assets => Occurences::NoneOrUpTo(None),
                OwnedRightsType::Prune => Occurences::NoneOrUpTo(None)
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
                    OwnedRightsType::Prune => Occurences::NoneOrUpTo(None),
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
            TransitionType::Prune => TransitionSchema {
                metadata: type_map! {
                    FieldType::PruneProof => Occurences::NoneOrUpTo(None)
                },
                closes: type_map! {
                    OwnedRightsType::Prune => Occurences::OnceOrUpTo(None),
                    OwnedRightsType::Assets => Occurences::OnceOrUpTo(None)
                },
                owned_rights: type_map! {
                    OwnedRightsType::Prune => Occurences::NoneOrUpTo(None),
                    OwnedRightsType::Assets => Occurences::NoneOrUpTo(None)
                },
                public_rights: Default::default(),
                abi: bmap! {}
            }
        },
    }
}

impl Neg for FieldType {
    type Output = usize;

    fn neg(self) -> Self::Output {
        self.to_usize().unwrap()
    }
}

impl Neg for OwnedRightsType {
    type Output = usize;

    fn neg(self) -> Self::Output {
        self.to_usize().unwrap()
    }
}

impl Neg for TransitionType {
    type Output = usize;

    fn neg(self) -> Self::Output {
        self.to_usize().unwrap()
    }
}
