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
    script, AssignmentAction, Bits, DataFormat, DiscreteFiniteFieldFormat, GenesisSchema,
    Occurences, Schema, StateFormat, StateSchema, TransitionSchema,
};

use crate::error::ServiceErrorDomain;
use crate::type_map;

impl From<SchemaError> for ServiceErrorDomain {
    fn from(err: SchemaError) -> Self {
        ServiceErrorDomain::Schema(format!("{}", err))
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Debug)]
#[repr(u16)]
pub enum FieldType {
    Name = 0,
    Description = 1,
    MaxIssues = 2,
    Timestamp = 3,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Debug)]
#[repr(u16)]
pub enum AssignmentsType {
    Issue = 0,
    Ownership = 1,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Debug)]
#[repr(u16)]
pub enum TransitionType {
    Issue = 0,
    Transfer = 1,
}

pub fn schema() -> Schema {
    Schema {
        field_types: type_map! {
            FieldType::Name => DataFormat::String(256),
            FieldType::Description => DataFormat::String(1024),
            FieldType::MaxIssues => DataFormat::Unsigned(Bits::Bit64, 0, core::u16::MAX as u128),
            // While UNIX timestamps allow negative numbers; in context of RGB Schema, assets
            // can't be issued in the past before RGB or Bitcoin even existed; so we prohibit
            // all the dates before RGB release
            // TODO: Update lower limit with the first RGB release
            // Current lower time limit is 07/04/2020 @ 1:54pm (UTC)
            FieldType::Timestamp => DataFormat::Integer(Bits::Bit64, 1593870844, core::i64::MAX as i128)
        },
        assignment_types: type_map! {
            AssignmentsType::Issue => StateSchema {
                format: DataFormat::Bytes(core::u16::MAX),
                abi: bmap! {}
            },
            AssignmentsType::Ownership => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {}
            }
        },
        genesis: GenesisSchema {
            metadata: type_map! {
                FieldType::Name => Occurences::Once,
                FieldType::Description => Occurences::NoneOrOnce,
                FieldType::MaxIssues => Occurences::NoneOrOnce,
                FieldType::Timestamp => Occurences::Once
            },
            defines: type_map! {
                AssignmentsType::Issue => Occurences::NoneOrOnce,
                AssignmentsType::Ownership => Occurences::NoneOrUpTo(None)
            },
            abi: bmap! {},
        },
        transitions: type_map! {
            TransitionType::Issue => TransitionSchema {
                metadata: type_map! {},
                closes: type_map! {
                    AssignmentsType::Issue => Occurences::Once
                },
                defines: type_map! {
                    AssignmentsType::Issue => Occurences::NoneOrOnce,
                    AssignmentsType::Ownership => Occurences::NoneOrUpTo(None)
                },
                abi: bmap! {
                    AssignmentAction::Validate => script::Procedure::Standard(script::StandardProcedure::CollectibleIssue)
                }
            },
            TransitionType::Transfer => TransitionSchema {
                metadata: type_map! {},
                closes: type_map! {
                    AssignmentsType::Ownership => Occurences::Once
                },
                defines: type_map! {
                    AssignmentsType::Ownership => Occurences::Once
                },
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
