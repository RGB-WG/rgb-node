// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#![recursion_limit = "256"]

//! Command-line interface to storm node

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

mod command;
mod opts;

use clap::Parser;
use colored::Colorize;
use microservices::shell::{Exec, LogLevel};
use rgb_rpc::client::{Client, Config};

pub use crate::opts::{Command, Opts};

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Config {
            rpc_endpoint: opts.rpc_endpoint,
            verbose: opts.verbose,
        }
    }
}

fn main() {
    println!("rgb-cli: command-line tool for working with RGB node");

    let opts = Opts::parse();
    LogLevel::from_verbosity_flag_count(opts.verbose).apply();
    trace!("Command-line arguments: {:#?}", &opts);

    let config: Config = opts.clone().into();
    trace!("Tool configuration: {:#?}", &config);

    let mut client = Client::with(config).expect("Error initializing client");

    trace!("Executing command: {}", opts.command);
    opts.exec(&mut client).unwrap_or_else(|err| {
        eprintln!("{} {}\n", "Error:".bright_red(), err.to_string().replace(": ", "\n  > ").red())
    });
}
