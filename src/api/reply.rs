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

use amplify::Wrapper;
use core::any::Any;
use std::io;
use std::sync::Arc;

use lnpbp::lnp::presentation::message::{Type, TypedEnum, Unmarshaller};
use lnpbp::lnp::presentation::{Error, UnknownTypeError, UnmarshallFn};
use lnpbp::strict_encoding::{strict_encode, StrictDecode};

const TYPE_OK: u16 = 1;
const TYPE_ERR: u16 = 0;

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub enum Reply {
    Success,
    Failure(String),
}

impl TypedEnum for Reply {
    fn try_from_type(type_id: Type, data: &dyn Any) -> Result<Self, UnknownTypeError> {
        Ok(match type_id.into_inner() {
            TYPE_OK => Self::Success,
            TYPE_ERR => Self::Failure(
                data.downcast_ref::<String>()
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
            Reply::Success => TYPE_OK,
            Reply::Failure(_) => TYPE_ERR,
        })
    }

    fn get_payload(&self) -> Vec<u8> {
        match self {
            Reply::Success => vec![],
            Reply::Failure(details) => {
                strict_encode(details).expect("Strict encoding for string has failed")
            }
        }
    }
}

impl Reply {
    pub fn create_unmarshaller() -> Unmarshaller<Self> {
        Unmarshaller::new(bmap! {
            TYPE_OK => Self::parse_success as UnmarshallFn<_>,
            TYPE_ERR => Self::parse_failure as UnmarshallFn<_>
        })
    }

    fn parse_success(_: &mut dyn io::Read) -> Result<Arc<dyn Any>, Error> {
        struct NoData;
        Ok(Arc::new(NoData))
    }

    fn parse_failure(mut reader: &mut dyn io::Read) -> Result<Arc<dyn Any>, Error> {
        Ok(Arc::new(String::strict_decode(&mut reader)?))
    }
}
