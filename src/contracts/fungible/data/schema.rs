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

use lnpbp::rgb::schema::{
    script, Bits, DataFormat, GenesisSchema, HomomorphicFormat, Occurences, Schema, Scripting,
    StateFormat, TransitionSchema,
};

use crate::error::ServiceErrorDomain;
use crate::type_map;

#[derive(Debug, Display, Error, From)]
#[display_from(Display)]
pub enum SchemaError {
    #[derive_from(core::option::NoneError)]
    NotAllFieldsPresent,
}

impl From<SchemaError> for ServiceErrorDomain {
    fn from(err: SchemaError) -> Self {
        ServiceErrorDomain::Schema(format!("{}", err))
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Display)]
#[repr(u8)]
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
#[display_from(Display)]
pub enum AssignmentsType {
    Issue = 0,
    Assets = 1,
    Prune = 2,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Display)]
pub enum TransitionType {
    Issue = 0,
    Transfer = 1,
    Prune = 2,
}

pub fn schema() -> Schema {
    Schema {
        field_types: type_map! {
            FieldType::Ticker => DataFormat::String(16),
            FieldType::Name => DataFormat::String(256),
            FieldType::Description => DataFormat::String(1024),
            FieldType::TotalSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::Precision => DataFormat::Unsigned(Bits::Bit64, 0, 18u128),
            FieldType::IssuedSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::DustLimit => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::PruneProof => DataFormat::Bytes(core::u16::MAX),
            FieldType::Timestamp => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128)
        },
        assignment_types: type_map! {
            AssignmentsType::Issue => StateFormat::Void,
            AssignmentsType::Assets => StateFormat::Homomorphic(HomomorphicFormat::Amount),
            AssignmentsType::Prune => StateFormat::Void
        },
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
            defines: type_map! {
                AssignmentsType::Issue => Occurences::NoneOrOnce,
                AssignmentsType::Assets => Occurences::NoneOrUpTo(None),
                AssignmentsType::Prune => Occurences::NoneOrUpTo(None)
            },
            scripting: Scripting {
                validation: script::Procedure::Standard(script::StandardProcedure::IssueControl),
                extensions: script::Extensions::ScriptsDenied,
            },
        },
        transitions: type_map! {
            TransitionType::Issue => TransitionSchema {
                metadata: type_map! {
                    FieldType::IssuedSupply => Occurences::Once
                },
                closes: type_map! {
                    AssignmentsType::Issue => Occurences::Once
                },
                defines: type_map! {
                    AssignmentsType::Issue => Occurences::NoneOrOnce,
                    AssignmentsType::Prune => Occurences::NoneOrUpTo(None),
                    AssignmentsType::Assets => Occurences::NoneOrUpTo(None)
                },
                scripting: Scripting {
                    validation: script::Procedure::Standard(script::StandardProcedure::IssueControl),
                    extensions: script::Extensions::ScriptsDenied,
                }
            },
            TransitionType::Transfer => TransitionSchema {
                metadata: type_map! {},
                closes: type_map! {
                    AssignmentsType::Assets => Occurences::OnceOrUpTo(None)
                },
                defines: type_map! {
                    AssignmentsType::Assets => Occurences::NoneOrUpTo(None)
                },
                scripting: Scripting {
                    validation: script::Procedure::Standard(script::StandardProcedure::ConfidentialAmount),
                    extensions: script::Extensions::ScriptsDenied,
                }
            },
            TransitionType::Prune => TransitionSchema {
                metadata: type_map! {
                    FieldType::PruneProof => Occurences::NoneOrUpTo(None)
                },
                closes: type_map! {
                    AssignmentsType::Prune => Occurences::OnceOrUpTo(None),
                    AssignmentsType::Assets => Occurences::OnceOrUpTo(None)
                },
                defines: type_map! {
                    AssignmentsType::Prune => Occurences::NoneOrUpTo(None),
                    AssignmentsType::Assets => Occurences::NoneOrUpTo(None)
                },
                scripting: Scripting {
                    // These means that the issuers may introduce custom
                    // prune validation procedure
                    validation: script::Procedure::NoValidation,
                    extensions: script::Extensions::ScriptsReplace,
                }
            }
        },
        script_library: vec![],
        script_extensions: script::Extensions::ScriptsDenied,
    }
}

impl Neg for FieldType {
    type Output = usize;

    fn neg(self) -> Self::Output {
        self.to_usize().unwrap()
    }
}

impl Neg for AssignmentsType {
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
