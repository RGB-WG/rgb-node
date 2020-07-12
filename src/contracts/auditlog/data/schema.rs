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
    Data = 2,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Debug)]
#[repr(u16)]
pub enum AssignmentsType {
    Entry = 0,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Debug)]
#[repr(u16)]
pub enum TransitionType {
    Entry = 0,
}

pub fn schema() -> Schema {
    Schema {
        field_types: type_map! {
            // Human-readable name for UI
            FieldType::Name => DataFormat::String(256),
            FieldType::Format => DataFormat::Unsigned(Bits::Bit16, 0, core::u16::MAX as u128),
            FieldType::Data => DataFormat::Bytes(core::u16::MAX),
            // While UNIX timestamps allow negative numbers; in context of RGB Schema, assets
            // can't be issued in the past before RGB or Bitcoin even existed; so we prohibit
            // all the dates before RGB release
            // TODO: Update lower limit with the first RGB release
            // Current lower time limit is 07/04/2020 @ 1:54pm (UTC)
            FieldType::StartsFrom => DataFormat::Integer(Bits::Bit64, 1593870844, core::i64::MAX as i128)
        },
        assignment_types: type_map! {
            AssignmentsType::Entry => StateSchema {
                format: StateFormat::Declarative,
                abi: bmap! {}
            },
        },
        genesis: GenesisSchema {
            metadata: type_map! {
                FieldType::Name => Occurences::Once,
                FieldType::Format => Occurences::Once,
                FieldType::Data => Occurences::Once,
                FieldType::StartsFrom => Occurences::Once
            },
            defines: type_map! {
                AssignmentsType::Entry => Occurences::NoneOrOnce
            },
            abi: bmap! {},
        },
        transitions: type_map! {
            TransitionType::Entry => TransitionSchema {
                metadata: type_map! {
                    FieldType::Format => Occurences::Once,
                    FieldType::Data => Occurences::Once,
                },
                closes: type_map! {
                    AssignmentsType::Entry => Occurences::Once,
                },
                defines: type_map! {
                    AssignmentsType::Entry => Occurences::NoneOrOnce,
                },
                abi: bmap! { }
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
