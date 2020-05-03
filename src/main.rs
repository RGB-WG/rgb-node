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

// We need this since code is not completed and a lot of it is written
// for future functionality
// Remove this once the first version will be complete
#![allow(dead_code)]
#![allow(unused_variables)]
// In mutithread environments it's critical to capture all failures
#![deny(unused_must_use)]
#![feature(never_type)]
#![feature(unwrap_infallible)]
#![feature(in_band_lifetimes)]
#![feature(try_trait)]
#![feature(with_options)]

extern crate clap;
extern crate diesel;
extern crate futures;
extern crate tokio;
extern crate zmq;
#[macro_use]
extern crate derive_wrapper;
#[macro_use]
extern crate async_trait;
extern crate chrono;
extern crate dotenv;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate num_derive;
extern crate num_traits;
extern crate rand;
extern crate regex;
extern crate rpassword;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate shellexpand;

extern crate electrum_client;
#[macro_use]
extern crate lnpbp;
#[macro_use]
mod rgbkit;

mod commands;
mod config;
mod constants;
mod data;
mod error;
mod runtime;

mod accounts;

use clap::derive::Clap;
use log::*;
use std::{env, fs};

use accounts::*;
use commands::*;
use config::*;
use error::Error;
use runtime::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // TODO: Parse config file as well
    let opts: Opts = Opts::parse();
    let config: Config = opts.clone().into();

    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            match config.verbose {
                0 => "error",
                1 => "warn",
                2 => "info",
                3 => "debug",
                4 => "trace",
                _ => "trace",
            },
        );
    }
    env_logger::init();
    log::set_max_level(LevelFilter::Trace);

    if let Command::Init = opts.command {
        if config.data_path(DataItem::Root).exists() {
            return Err(Error::from(format!(
                "Data directory {:?} already initialized, exiting",
                config.data_dir
            )));
        }

        let password =
            rpassword::prompt_password_stderr("Password for private keys vault encryption: ")?;
        if !(8..256).contains(&password.len()) {
            return Err(Error::from(
                "The length of the password must be at least 8 and no more than 256 characters",
            ));
        }

        info!("Generating seed phrase ...");
        fs::create_dir_all(config.data_dir.clone())?;
        KeyringManager::setup(config.data_path(DataItem::KeyringVault), &password)?;
        return Ok(());
    }

    let runtime = Runtime::init(config.clone()).await?;

    // Non-interactive command processing:
    debug!("Parsing and processing a command");
    match opts.command {
        Command::Account(subcommand) => match subcommand {
            account::Command::List => runtime.account_list(),
            account::Command::Create {
                name,
                derivation_path,
                description,
            } => runtime.account_create(name, derivation_path, description.unwrap_or_default()),
            account::Command::DepositBoxes {
                no,
                offset,
                account,
            } => runtime.account_deposit_boxes(account, offset, no),
        },
        Command::Bitcoin(subcommand) => match subcommand {
            bitcoin::Command::Funds {
                no,
                offset,
                deposit_types,
                account,
            } => {
                runtime
                    .bitcoin_funds(account, deposit_types, offset, no)
                    .await
            }
        },
        Command::Fungible(subcommand) => Ok(subcommand.exec(&config)?),
        /*match subcommand {
            fungible::Command::List =>
                runtime.fungible_list(),
            fungible::Command::Funds { account, contract_id, only_owned, deposit_types } =>
                runtime.fungible_funds(account, deposit_types, contract_id, only_owned).await,
            fungible::Command::Issue(issue) =>
                runtime.fungible_issue(issue),
            fungible::Command::Pay(payment) =>
                runtime.fungible_pay(payment).await,
            _ => unimplemented!()
        }
         */
        //Command::Query { query } => runtime.command_query(query).await?,
        _ => unimplemented!(),
    }
}
