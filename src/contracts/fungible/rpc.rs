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
use lnpbp::strict_encoding::{strict_encode, StrictDecode};
use lnpbp::Wrapper;

use crate::api::fungible::{Issue, Transfer};

const TYPE_ISSUE: u16 = 1000;

#[derive(Clone, PartialEq, Debug, Display)]
#[display_from(Debug)]
#[non_exhaustive]
pub enum Command {
    Issue(Issue),
    Transfer(Transfer),
    //Receive(Receive),
}

impl TypedEnum for Command {
    fn try_from_type(type_id: Type, data: &dyn Any) -> Result<Self, UnknownTypeError> {
        Ok(match type_id.into_inner() {
            TYPE_ISSUE => Self::Issue(
                data.downcast_ref::<Issue>()
                    .expect("Internal API parser inconsistency")
                    .clone(),
            ),
            _ => Err(UnknownTypeError)?,
        })
    }

    fn get_type(&self) -> Type {
        Type::from_inner(match self {
            Command::Issue(_) => TYPE_ISSUE,
            _ => unimplemented!(),
        })
    }

    fn get_payload(&self) -> Vec<u8> {
        match self {
            Command::Issue(issue) => {
                strict_encode(issue).expect("Strict encoding for issue structure has failed")
            }
            _ => unimplemented!(),
        }
    }
}

impl Command {
    pub fn create_unmarshaller() -> Unmarshaller<Self> {
        Unmarshaller::new(bmap! {
            TYPE_ISSUE => Self::parse_issue as UnmarshallFn<_>
        })
    }

    fn parse_issue(mut reader: &mut dyn io::Read) -> Result<Arc<dyn Any>, Error> {
        Ok(Arc::new(Issue::strict_decode(&mut reader)?))
    }
}
