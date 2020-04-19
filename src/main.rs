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
#![feature(repr_transparent)]

#[macro_use]
extern crate tokio;
extern crate futures;
extern crate zmq;
#[macro_use]
extern crate diesel;
extern crate clap;
#[macro_use]
extern crate derive_wrapper;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate dotenv;
extern crate chrono;
extern crate rand;
extern crate rpassword;
extern crate shellexpand;

extern crate lnpbp;
extern crate rgb;


mod constants;
mod error;
mod config;
mod commands;
mod runtime;

mod accounts;


use std::{env, fs};
use log::*;
use clap::derive::Clap;

use config::*;
use runtime::*;
use commands::*;
use error::Error;
use std::process::exit;
use accounts::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // TODO: Parse config file as well
    let opts: Opts = Opts::parse();
    let config: Config = opts.clone().into();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", match config.verbose {
            0 => "error",
            1 => "warn",
            2 => "info",
            3 => "debug",
            4 => "trace",
            _ => "trace",
        });
    }
    env_logger::init();
    log::set_max_level(LevelFilter::Trace);

    if let Command::Init = opts.command {
        if config.data_path(DataItem::Root).exists() {
            return Err(Error::from(format!("Data directory {:?} already initialized, exiting", config.data_dir)));
        }

        let password = rpassword::prompt_password_stderr("Password for private keys vault encryption: ")?;
        if !(8..256).contains(&password.len()) {
            return Err(Error::from("The length of the password must be at least 8 and no more than 256 characters"));
        }

        info!("Generating seed phrase ...");
        fs::create_dir_all(config.data_dir.clone())?;
        KeyringManager::setup(config.data_path(DataItem::KeyringVault), &password)?;
        return Ok(())
    }

    let runtime = Runtime::init(config).await?;

    // Non-interactive command processing:
    debug!("Parsing and processing a command");
    match opts.command {
        //Command::Query { query } => runtime.command_query(query).await?,
        _ => unimplemented!()
    }

    Ok(())
}
