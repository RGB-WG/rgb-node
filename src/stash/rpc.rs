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

use core::any::Any;
use std::io;
use std::sync::Arc;

use lnpbp::lnp::presentation::{Error, UnknownTypeError};
use lnpbp::lnp::{Type, TypedEnum, UnmarshallFn, Unmarshaller};
use lnpbp::rgb::{Consignment, ContractId, Genesis, Schema, SchemaId, TransitionId};
use lnpbp::strict_encoding::{strict_encode, StrictDecode};
use lnpbp::Wrapper;

use crate::api::stash::ConsignRequest;

const TYPE_ADD_GENESIS: u16 = 1000;

#[derive(Clone, Debug, Display)]
#[display_from(Debug)]
#[non_exhaustive]
pub enum Command {
    AddSchema(Schema),
    //ListSchemata(),
    //ReadSchemata(Vec<SchemaId>),
    AddGenesis(Genesis),
    //ListGeneses(),
    //ReadGeneses(Vec<ContractId>),

    //ReadTransitions(Vec<TransitionId>),
    Consign(ConsignRequest),
    MergeConsignment(Consignment),
    VerifyConsignment(Consignment),
    ForgetConsignment(Consignment),
}

impl TypedEnum for Command {
    fn try_from_type(type_id: Type, data: &dyn Any) -> Result<Self, UnknownTypeError> {
        Ok(match type_id.into_inner() {
            TYPE_ADD_GENESIS => Self::AddGenesis(
                data.downcast_ref::<Genesis>()
                    .expect("Internal API parser inconsistency")
                    .clone(),
            ),
            // Here we receive odd-numbered messages. However, in terms of RPC,
            // there is no "upstream processor", so we return error (but do not
            // break connection).
            _ => Err(UnknownTypeError)?,
        })
    }

    fn get_type(&self) -> Type {
        Type::from_inner(match self {
            Command::AddGenesis(_) => TYPE_ADD_GENESIS,
            _ => unimplemented!(),
        })
    }

    fn get_payload(&self) -> Vec<u8> {
        match self {
            Command::AddGenesis(genesis) => {
                strict_encode(genesis).expect("Strict encoding for genesis has failed")
            }
            _ => unimplemented!(),
        }
    }
}

impl Command {
    pub fn create_unmarshaller() -> Unmarshaller<Self> {
        Unmarshaller::new(bmap! {
            TYPE_ADD_GENESIS => Self::parse_genesis as UnmarshallFn<_>
        })
    }

    fn parse_genesis(mut reader: &mut dyn io::Read) -> Result<Arc<dyn Any>, Error> {
        Ok(Arc::new(Genesis::strict_decode(&mut reader)?))
    }
}
