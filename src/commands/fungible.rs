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


use std::{path::PathBuf, str::FromStr};
use clap::Clap;
use lnpbp::bitcoin::{TxIn, OutPoint};
use lnpbp::{bp, rgb};
use ::rgb::fungible;


/// Defines information required to generate bitcoin transaction output from
/// command-line argument
#[derive(Clone, Debug, Display)]
#[display_from(Debug)]
pub struct Output {
    pub amount: bitcoin::Amount,
    pub lock: bp::PubkeyScriptSource,
}

impl FromStr for Output {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

/// Defines information required to generate bitcoin transaction input from
/// command-line argument
#[derive(Clone, Debug, Display)]
#[display_from(Debug)]
pub struct Input {
    pub txin: TxIn,
    pub unlock: bp::PubkeyScriptSource,
}

impl FromStr for Input {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}


#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
    /// Lists all known asset ids
    List {
        /// Include only the assets which are owned by the known accounts
        #[clap(short, long)]
        owned: bool
    },

    /// Creates a new asset
    Issue {
        /// Limit for the total supply (by default equals to the `amount`
        #[clap(short, long)]
        supply: Option<rgb::data::Amount>,

        /// Enables secondary issuance/inflation; takes UTXO seal definition
        /// as its value
        #[clap(short, long, requires("supply"))]
        inflatable: Option<OutPoint>,

        /// Precision, i.e. number of digits reserved for fractional part
        #[clap(short, long, default_value="0")]
        precision: u8,

        /// Dust limit for asset transfers; defaults to no limit
        #[clap(short="D", long, default_value="0")]
        dust_limit: rgb::data::Amount,

        /// Filename to save asset genesis to; defaults to `ticker.rgb`
        #[clap(short, long)]
        output: Option<PathBuf>,

        /// Asset ticker (will be capitalized)
        ticker: String,

        /// Asset title
        title: String,

        /// Asset description
        #[clap(short, long)]
        description: Option<String>,

        /// Asset allocation, in form of <amount>@<txid>:<vout>
        #[clap(min_values=1)]
        allocate: Vec<fungible::Allocation>,
    },

    /// Transfers some asset to another party
    Pay {
        /// Use custom commitment output for generated witness transaction
        #[clap(long)]
        commit_txout: Option<Output>,

        /// Adds output(s) to generated witness transaction
        #[clap(long)]
        txout: Vec<Output>,

        /// Adds input(s) to generated witness transaction
        #[clap(long)]
        txin: Vec<Input>,

        /// Allocates other assets to custom outputs
        #[clap(short, long)]
        allocate: Vec<fungible::Allocation>,

        /// Saves witness transaction to a file instead of publishing it
        #[clap(short, long)]
        transaction: Option<PathBuf>,

        /// Saves proof data to a file instead of sending it to the remote party
        #[clap(short, long)]
        proof: Option<PathBuf>,

        /// Invoice to pay
        invoice: fungible::Invoice,

        /// Overrides amount provided in the invoice
        amount: rgb::data::Amount,
    }
}
