// RGB Node: sovereign smart contracts backend
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
// Written in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2020-2024 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2025 RGB Consortium, Switzerland. All rights reserved.
// Copyright (C) 2020-2025 Dr Maxim Orlovsky. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

#[macro_use]
extern crate clap;

mod opts;

use std::fs;
use std::process::{ExitCode, Termination, exit};

use clap::Parser;
use loglevel::LogLevel;
use rgb::Consensus;
use rgb::popls::bp::seals::TxoSeal;
use rgb_persist_fs::StockpileDir;
pub use rgbnode;
use rgbnode::{Broker, BrokerError, DbHolder};

use crate::opts::{Command, Opts};

struct Status(Result<(), BrokerError>);

impl Termination for Status {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("Error: {err}");
                ExitCode::FAILURE
            }
        }
    }
}

fn main() -> Status {
    let mut opts = Opts::parse();
    opts.process();
    LogLevel::from_verbosity_flag_count(opts.verbose).apply();
    log::debug!("Command-line arguments: {:#?}", &opts);

    let (conf, command) = opts.into_config_cmd();
    let data_dir = &conf.data_dir;
    match command {
        Some(Command::Init) => {
            eprint!("Initializing ... ");
            if let Err(err) = fs::create_dir_all(data_dir) {
                eprintln!("unable to create data directory at '{}'\n{err}", data_dir.display());
                exit(3);
            }
            if let Err(err) = DbHolder::init(data_dir) {
                eprintln!("unable to create wallet database at '{}'\n{err}", data_dir.display());
                exit(4);
            }
            eprintln!("done");
            Status(Ok(()))
        }
        None => {
            let stockpile = StockpileDir::<TxoSeal>::load(
                data_dir.clone(),
                Consensus::Bitcoin,
                conf.network.is_testnet(),
            )
            .unwrap_or_else(|err| {
                eprintln!("Can't load stockpile from '{}' {err}", data_dir.display());
                exit(5);
            });
            let status = Broker::run_standalone(conf, stockpile);
            Status(status)
        }
    }
}
