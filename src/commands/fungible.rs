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

use bech32::{self, ToBase32};
use clap::Clap;
use regex::Regex;
use std::path::PathBuf;
use std::sync::Arc;

use bitcoin::hashes::hex::FromHex;
use bitcoin::TxIn;

use lnpbp::bitcoin;
use lnpbp::bp;
use lnpbp::rgb::prelude::*;
use lnpbp::strict_encoding::strict_encode;

use crate::commands::bitcoin::DepositType;
use crate::config::{Config, DataItem};

use crate::rgbkit::fungible::{IssueStructure, Manager, Outcoins};
use crate::rgbkit::{self, fungible, DiskStorage, DiskStorageConfig, InteroperableError, SealSpec};

#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
    /// Lists all known assets
    List,

    /// Lists all known funds for a given asset
    Funds {
        /// Include only the assets which are owned by the known accounts
        #[clap(short, long)]
        only_owned: bool,

        /// Tag name of the account to list deposit boxes
        account: String,

        /// Assets
        #[clap(parse(try_from_str=ContractId::from_hex))]
        contract_id: ContractId,

        /// Request funds on the specified deposit types only
        #[clap(arg_enum)]
        #[clap(default_value = "WPKH")]
        deposit_types: Vec<DepositType>,
    },

    /// Creates a new asset
    Issue(Issue),

    /// Transfers some asset to another party
    Pay(Pay),
}

impl Command {
    pub fn exec(self, global: &Config) -> Result<(), InteroperableError> {
        let rgb_storage = DiskStorage::new(DiskStorageConfig {
            data_dir: global.data_path(DataItem::Root),
        })?;
        let asset_storage = fungible::DiskStorage {};

        let manager = Manager::new(Arc::new(rgb_storage), Arc::new(asset_storage))?;

        match self {
            Command::List => unimplemented!(),
            Command::Funds { .. } => unimplemented!(),
            Command::Issue(issue) => issue.exec(global, &manager),
            Command::Pay(_) => unimplemented!(),
        }
    }
}

#[derive(Clap, Clone, Debug, Display)]
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

impl Issue {
    pub fn exec(
        self,
        global: &Config,
        manager: &fungible::Manager,
    ) -> Result<(), InteroperableError> {
        let issue_structure = match self.inflatable {
            None => IssueStructure::SingleIssue,
            Some(seal_spec) => IssueStructure::MultipleIssues {
                max_supply: self.supply.expect("Clap is broken"),
                reissue_control: seal_spec,
            },
        };

        info!("Issuing asset ...");
        let (asset, genesis) = manager.issue(
            global.network,
            self.ticker,
            self.title,
            self.description,
            issue_structure,
            self.allocate,
            self.precision,
            vec![], // we do not support pruning yet
            self.dust_limit,
        )?;

        debug!("Asset information:\n {}\n", asset);
        trace!("Genesis contract:\n {}\n", genesis);

        let bech = bech32::encode(
            rgbkit::RGB_BECH32_HRP_GENESIS,
            strict_encode(&genesis).to_base32(),
        )
        .unwrap();
        info!(
            "Use this string to send information about the issued asset:\n{}\n",
            bech
        );
        Ok(())
    }
}

#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub struct Pay {
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

    /// Tag name of the account for controlling transaciton outputs
    pub account: String,

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
    #[derive(Clone, Debug, Display)]
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
    #[derive(Clone, Debug, Display)]
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
