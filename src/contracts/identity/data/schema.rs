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

use crate::type_map;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Debug)]
#[repr(u16)]
pub enum FieldType {
    Name = 0,
    Format = 1,
    Identity = 2,
    Cryptography = 3,
    PublicKey = 4,
    Signature = 5,
    ValidFrom = 6,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Debug)]
#[repr(u16)]
pub enum AssignmentsType {
    Revocation = 0,
    Extension = 1,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Debug)]
#[repr(u16)]
pub enum TransitionType {
    Identity = 0,
}

pub fn schema() -> Schema {
    Schema {
        field_types: type_map! {
            FieldType::Name => DataFormat::String(256),
            FieldType::Format => DataFormat::Unsigned(Bits::Bit16, 0, core::u16::MAX as u128),
            FieldType::Identity => DataFormat::Bytes(core::u16::MAX),
            FieldType::Cryptography => DataFormat::Unsigned(Bits::Bit16, 0, core::u16::MAX as u128),
            FieldType::PublicKey => DataFormat::Bytes(core::u16::MAX),
            FieldType::Signature => DataFormat::Bytes(core::u16::MAX),
            // While UNIX timestamps allow negative numbers; in context of RGB Schema, assets
            // can't be issued in the past before RGB or Bitcoin even existed; so we prohibit
            // all the dates before RGB release
            // TODO: Update lower limit with the first RGB release
            // Current lower time limit is 07/04/2020 @ 1:54pm (UTC)
            FieldType::ValidFrom => DataFormat::Integer(Bits::Bit64, 1593870844, core::i64::MAX as i128)
        },
        assignment_types: type_map! {
            AssignmentsType::Revocation => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {}
            },
            AssignmentsType::Extension => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {}
            }
        },
        genesis: GenesisSchema {
            metadata: type_map! {
                FieldType::Name => Occurences::Once,
                FieldType::Format => Occurences::Once,
                FieldType::Identity => Occurences::Once,
                FieldType::Cryptography => Occurences::Once,
                FieldType::PublicKey => Occurences::Once,
                FieldType::Signature => Occurences::Once,
                FieldType::ValidFrom => Occurences::Once
            },
            defines: type_map! {
                AssignmentsType::Revocation => Occurences::Once,
                AssignmentsType::Extension => Occurences::NoneOrUpTo(None)
            },
            abi: bmap! {
                AssignmentAction::Validate => script::Procedure::Standard(script::StandardProcedure::IdentityValidate)
            },
        },
        transitions: type_map! {
            TransitionType::Identity => TransitionSchema {
                metadata: type_map! {
                    FieldType::Name => Occurences::Once,
                    FieldType::Format => Occurences::Once,
                    FieldType::Identity => Occurences::Once,
                    FieldType::Cryptography => Occurences::Once,
                    FieldType::PublicKey => Occurences::Once,
                    FieldType::Signature => Occurences::Once,
                    FieldType::ValidFrom => Occurences::Once
                },
                closes: type_map! {
                    AssignmentsType::Revocation => Occurences::NoneOrUpTo(None),
                    AssignmentsType::Extension => Occurences::NoneOrUpTo(None),
                },
                defines: type_map! {
                    AssignmentsType::Revocation => Occurences::Once,
                    AssignmentsType::Extension => Occurences::NoneOrUpTo(None)
                },
                abi: bmap! {
                    AssignmentAction::Validate => script::Procedure::Standard(script::StandardProcedure::IdentityValidate)
                }
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
