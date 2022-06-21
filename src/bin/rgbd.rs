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

use clap::Parser;
use microservices::error::BootstrapError;
use rgb_node::rgbd::Opts;
use rgb_node::{rgbd, Config, LaunchError};

fn main() -> Result<(), BootstrapError<LaunchError>> {
    println!("rgbd: RGB stash microservice");

    let mut opts = Opts::parse();
    trace!("Command-line arguments: {:?}", opts);
    opts.process();
    trace!("Processed arguments: {:?}", opts);

    let config = Config::from(opts);
    trace!("Daemon configuration: {:?}", config);
    debug!("CTL socket {}", config.ctl_endpoint);
    debug!("RPC socket {}", config.rpc_endpoint);
    debug!("STORE socket {}", config.store_endpoint);
    debug!("STORM socket {}", config.storm_endpoint);

    /*
    use self::internal::ResultExt;
    let (config_from_file, _) =
        internal::Config::custom_args_and_optional_files(std::iter::empty::<
            &str,
        >())
        .unwrap_or_exit();
     */

    debug!("Starting runtime ...");
    rgbd::run(config).expect("running rgbd runtime");

    unreachable!()
}
