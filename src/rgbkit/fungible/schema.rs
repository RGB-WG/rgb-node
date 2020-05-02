use super::*;
use crate::type_map;
use num_derive::{FromPrimitive, ToPrimitive};
use rgb::schema::{
    Bits, DataFormat, GenesisSchema, HomomorphicFormat, Occurences, Scripting, StateFormat,
    TransitionSchema,
};

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
    FractionalBits = 6,
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
            FieldType::IssuedSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::DustLimit => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
            FieldType::PruneProof => DataFormat::Bytes(core::u16::MAX),
            FieldType::Timestamp => DataFormat::Unsigned(Bits::Bit32, 0, core::u32::MAX as u128)
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
                FieldType::FractionalBits => Occurences::Once
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
                    FieldType::PruneProof => Occurences::OnceOrUpTo(None)
                },
                closes: type_map! {
                    AssignmentsType::Prune => Occurences::NoneOrUpTo(None),
                    AssignmentsType::Assets => Occurences::NoneOrUpTo(None)
                },
                defines: type_map! {
                    AssignmentsType::Prune => Occurences::NoneOrUpTo(None),
                    AssignmentsType::Assets => Occurences::NoneOrUpTo(None)
                },
                scripting: Scripting {
                    validation: script::Procedure::NoValidation,
                    extensions: script::Extensions::ScriptsReplace,
                }
            }
        },
        script_library: vec![],
        script_extensions: script::Extensions::ScriptsDenied,
    }
}
