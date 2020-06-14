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
use clap::Clap;
use core::any::Any;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io;
use std::sync::Arc;

use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::OutPoint;

use lnpbp::bitcoin;
use lnpbp::lnp::presentation::{Error, UnknownTypeError};
use lnpbp::lnp::{Type, TypedEnum, UnmarshallFn, Unmarshaller};
use lnpbp::rgb::prelude::*;
use lnpbp::strict_encoding::{strict_encode, StrictDecode};

use crate::fungible::{Outcoincealed, Outcoins};
use crate::util::SealSpec;

const TYPE_ISSUE: u16 = 1000;
const TYPE_TRANSFER: u16 = 1001;

#[derive(Clone, PartialEq, Debug, Display)]
#[display_from(Debug)]
#[non_exhaustive]
pub enum Request {
    Issue(Issue),
    Transfer(TransferApi),
    //Receive(Receive),
}

impl TypedEnum for Request {
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
            Request::Issue(_) => TYPE_ISSUE,
            _ => unimplemented!(),
        })
    }

    fn get_payload(&self) -> Vec<u8> {
        match self {
            Request::Issue(issue) => {
                strict_encode(issue).expect("Strict encoding for issue structure has failed")
            }
            _ => unimplemented!(),
        }
    }
}

impl Request {
    pub fn create_unmarshaller() -> Unmarshaller<Self> {
        Unmarshaller::new(bmap! {
            TYPE_ISSUE => Self::parse_issue as UnmarshallFn<_>,
            TYPE_TRANSFER => Self::parse_transfer as UnmarshallFn<_>
        })
    }

    fn parse_issue(mut reader: &mut dyn io::Read) -> Result<Arc<dyn Any>, Error> {
        Ok(Arc::new(Issue::strict_decode(&mut reader)?))
    }

    fn parse_transfer(mut reader: &mut dyn io::Read) -> Result<Arc<dyn Any>, Error> {
        Ok(Arc::new(TransferApi::strict_decode(&mut reader)?))
    }
}

#[derive(
    Clap, Clone, PartialEq, Serialize, Deserialize, StrictEncode, StrictDecode, Debug, Display,
)]
#[display_from(Debug)]
pub struct Issue {
    /// Asset ticker
    #[clap(validator=ticker_validator)]
    pub ticker: String,

    /// Asset title
    pub title: String,

    /// Asset description
    #[clap(short, long)]
    pub description: Option<String>,

    /// Limit for the total supply; ignored if the asset can't be inflated
    #[clap(short, long)]
    pub supply: Option<f32>,

    /// Enables secondary issuance/inflation; takes UTXO seal definition
    /// as its value
    #[clap(short, long, requires("supply"))]
    pub inflatable: Option<SealSpec>,

    /// Precision, i.e. number of digits reserved for fractional part
    #[clap(short, long, default_value = "0")]
    pub precision: u8,

    /// Dust limit for asset transfers; defaults to no limit
    #[clap(short = "D", long)]
    pub dust_limit: Option<Amount>,

    /// Asset allocation, in form of <amount>@<txid>:<vout>
    #[clap(required = true)]
    pub allocate: Vec<Outcoins>,
}

#[derive(Clone, PartialEq, StrictEncode, StrictDecode, Debug, Display)]
#[display_from(Debug)]
pub struct TransferApi {
    /// Asset contract id
    pub contract_id: ContractId,

    /// Base layer transaction structure to use
    pub psbt: PartiallySignedTransaction,

    /// Asset input: unspent transaction outputs
    pub inputs: Vec<OutPoint>,

    /// Asset change allocations
    ///
    /// Here we always know an explicit outpoint that will contain the assets
    pub ours: Vec<Outcoins>,

    /// Receiver's allocations.
    ///
    /// They are kept separate from change allocations since here we do not
    /// know the actual seals and only know hashes derived from seal data and
    /// blinding entropy.
    pub theirs: Vec<Outcoincealed>,

    /// Optional change output: the rest of assets will be allocated here
    pub change: Option<OutPoint>,
}

fn ticker_validator(name: &str) -> Result<(), String> {
    let re = Regex::new(r"^[A-Z]{3,8}$").expect("Regex parse failure");
    if !re.is_match(&name) {
        Err(
            "Ticker name must be between 2 and 8 chars, contain no spaces and \
            consist only of capital letters\
            "
            .to_string(),
        )
    } else {
        Ok(())
    }
}
