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
use std::sync::{Arc, Mutex};

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
    Transfer(Transfer),
}

impl Command {
    pub fn exec(self, global: &Config) -> Result<(), InteroperableError> {
        let mut data_dir = global.data_path(DataItem::Root);
        let rgb_storage = DiskStorage::new(DiskStorageConfig {
            data_dir: data_dir.clone(),
        })?;
        data_dir.push("fungible");
        let asset_storage = fungible::DiskStorage::new(fungible::DiskStorageConfig { data_dir })?;

        let mut manager = Manager::new(
            Arc::new(Mutex::new(rgb_storage)),
            Arc::new(Mutex::new(asset_storage)),
        )?;

        match self {
            Command::List => {
                println!("\nKnown assets:\n\n");
                manager
                    .assets()?
                    .iter()
                    .for_each(|asset| println!("{}", asset));
                Ok(())
            }
            Command::Funds { .. } => unimplemented!(),
            Command::Issue(issue) => issue.exec(global, &mut manager),
            Command::Pay(_) => unimplemented!(),
        }
    }
}

impl Issue {
    pub fn exec(
        self,
        global: &Config,
        manager: &mut fungible::Manager,
    ) -> Result<(), InteroperableError> {
        info!("Issuing asset ...");
        debug!("{}", self.clone());

        let issue_structure = match self.inflatable {
            None => IssueStructure::SingleIssue,
            Some(seal_spec) => IssueStructure::MultipleIssues {
                max_supply: self.supply.expect("Clap is broken"),
                reissue_control: seal_spec,
            },
        };

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
