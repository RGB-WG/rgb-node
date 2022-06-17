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

//! Main executable for RGB node.

#[macro_use]
extern crate log;

mod opts;

use clap::Parser;
use microservices::error::BootstrapError;
use microservices::shell::LogLevel;
use rgb_node::{rgbd, Config, LaunchError};

use crate::opts::Opts;

fn main() -> Result<(), BootstrapError<LaunchError>> {
    println!("rgbd: RGB stash microservice");

    let opts = Opts::parse();
    LogLevel::from_verbosity_flag_count(opts.verbose).apply();
    trace!("Command-line arguments: {:?}", &opts);

    let mut config = Config {
        data_dir: opts.data_dir,
        rpc_endpoint: opts.rpc_endpoint,
        storm_endpoint: opts.storm_endpoint,
        store_endpoint: opts.store_endpoint,
        verbose: opts.verbose,
    };
    trace!("Daemon configuration: {:?}", config);
    config.process();
    trace!("Processed configuration: {:?}", config);
    debug!("CTL RPC socket {}", config.rpc_endpoint);

    /*
    use self::internal::ResultExt;
    let (config_from_file, _) =
        internal::Config::custom_args_and_optional_files(std::iter::empty::<
            &str,
        >())
        .unwrap_or_exit();
     */

    debug!("Starting runtime ...");
    rgbd::service::run(config).expect("running rgbd runtime");

    unreachable!()
}
