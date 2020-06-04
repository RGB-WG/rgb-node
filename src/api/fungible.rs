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

use clap::Clap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

use bitcoin::consensus::encode::{Decodable, Encodable};
use bitcoin::hashes::hex::FromHex;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::TxIn;

use lnpbp::bitcoin;
use lnpbp::bp;
use lnpbp::rgb::prelude::*;
use lnpbp::strict_encoding::{Error, StrictDecode, StrictEncode};

use crate::fungible::Outcoins;
use crate::util::SealSpec;

#[derive(Clap, Clone, PartialEq, Serialize, Deserialize, Debug, Display)]
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

impl StrictEncode for Issue {
    type Error = Error;

    fn strict_encode<E: io::Write>(&self, mut e: E) -> Result<usize, Self::Error> {
        Ok(strict_encode_list!(e;
            self.ticker,
            self.title,
            self.description,
            self.supply,
            self.inflatable,
            self.precision,
            self.dust_limit,
            self.allocate
        ))
    }
}

impl StrictDecode for Issue {
    type Error = Error;

    fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Self::Error> {
        Ok(Self {
            ticker: String::strict_decode(&mut d)?,
            title: String::strict_decode(&mut d)?,
            description: Option::<String>::strict_decode(&mut d)?,
            supply: Option::<f32>::strict_decode(&mut d)?,
            inflatable: Option::<SealSpec>::strict_decode(&mut d)?,
            precision: u8::strict_decode(&mut d)?,
            dust_limit: Option::<Amount>::strict_decode(&mut d)?,
            allocate: Vec::<Outcoins>::strict_decode(&mut d)?,
        })
    }
}

#[derive(Clone, PartialEq, Debug, Display)]
#[display_from(Debug)]
pub struct TransferApi {
    /// Base layer transaction structure to use
    pub psbt: PartiallySignedTransaction,

    /// Allocates other assets to custom outputs
    pub allocate: Vec<Outcoins>,

    /// Amount
    pub amount: Amount,

    /// Assets
    pub contract_id: ContractId,

    /// Receiver
    pub receiver: bp::blind::OutpointHash,
}

impl StrictEncode for TransferApi {
    type Error = Error;

    fn strict_encode<E: io::Write>(&self, mut e: E) -> Result<usize, Self::Error> {
        self.psbt.consensus_encode(&mut e)?;
        Ok(strict_encode_list!(e;
            self.allocate,
            self.amount,
            self.contract_id,
            self.receiver
        ))
    }
}

impl StrictDecode for TransferApi {
    type Error = Error;

    fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Self::Error> {
        let psbt = PartiallySignedTransaction::consensus_decode(&mut d)?;
        Ok(Self {
            psbt,
            allocate: Vec::<Outcoins>::strict_decode(&mut d)?,
            amount: Amount::strict_decode(&mut d)?,
            contract_id: ContractId::strict_decode(&mut d)?,
            receiver: bp::blind::OutpointHash::strict_decode(&mut d)?,
        })
    }
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
