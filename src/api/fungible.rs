// Kaleidoscope: RGB command-line wallet utility
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Alekos Filini <alekos.filini@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use clap::Clap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

use bitcoin::hashes::hex::FromHex;
use bitcoin::TxIn;

use lnpbp::bitcoin;
use lnpbp::bp;
use lnpbp::rgb::prelude::*;

use crate::fungible::Outcoins;
use crate::util::SealSpec;
use lnpbp::strict_encoding::{Error, StrictDecode};

#[derive(Clap, Clone, PartialEq, Serialize, Deserialize, Debug, Display)]
#[display_from(Debug)]
pub struct Issue {
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

    /// Filename to export asset genesis to;
    /// saves into data dir if not provided
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Asset ticker
    #[clap(validator=ticker_validator)]
    pub ticker: String,

    /// Asset title
    pub title: String,

    /// Asset description
    #[clap(short, long)]
    pub description: Option<String>,

    /// Asset allocation, in form of <amount>@<txid>:<vout>
    #[clap(required = true)]
    pub allocate: Vec<Outcoins>,
}

impl StrictDecode for Issue {
    type Error = Error;

    fn strict_decode<D: io::Read>(d: D) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

#[derive(Clap, Clone, PartialEq, Debug, Display)]
#[display_from(Debug)]
pub struct Transfer {
    /// Use custom commitment output for generated witness transaction
    #[clap(long)]
    pub commit_txout: Option<Output>,

    /// Adds output(s) to generated witness transaction
    #[clap(long)]
    pub txout: Vec<Output>,

    /// Adds input(s) to generated witness transaction
    #[clap(long)]
    pub txin: Vec<Input>,

    /// Allocates other assets to custom outputs
    #[clap(short, long)]
    pub allocate: Vec<Outcoins>,

    /// Saves witness transaction to a file instead of publishing it
    #[clap(short, long)]
    pub transaction: Option<PathBuf>,

    /// Saves proof data to a file instead of sending it to the remote party
    #[clap(short, long)]
    pub proof: Option<PathBuf>,

    /// Amount
    pub amount: Amount,

    /// Assets
    #[clap(parse(try_from_str=ContractId::from_hex))]
    pub contract_id: ContractId,

    /// Receiver
    #[clap(parse(try_from_str=bp::blind::OutpointHash::from_hex))]
    pub receiver: bp::blind::OutpointHash,
    // / Invoice to pay
    //pub invoice: fungible::Invoice,
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

// Helper data structures

mod helpers {
    use super::*;
    use core::str::FromStr;

    /// Defines information required to generate bitcoin transaction output from
    /// command-line argument
    #[derive(Clone, PartialEq, Debug, Display)]
    #[display_from(Debug)]
    pub struct Output {
        pub amount: bitcoin::Amount,
        pub lock: bp::LockScript,
    }

    impl FromStr for Output {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            unimplemented!()
        }
    }

    /// Defines information required to generate bitcoin transaction input from
    /// command-line argument
    #[derive(Clone, PartialEq, Debug, Display)]
    #[display_from(Debug)]
    pub struct Input {
        pub txin: TxIn,
        pub unlock: bp::LockScript,
    }

    impl FromStr for Input {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            unimplemented!()
        }
    }
}
pub use helpers::*;
