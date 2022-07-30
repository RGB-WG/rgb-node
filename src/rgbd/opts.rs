// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use clap::{Parser, ValueHint};
use internet2::addr::ServiceAddr;
use rgb_rpc::RGB_NODE_RPC_ENDPOINT;
use storm_ext::STORM_NODE_EXT_ENDPOINT;

use crate::opts::Opts as SharedOpts;

/// Command-line arguments
#[derive(Parser)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[clap(author, version, name = "rgbd", about = "RGB node managing service")]
pub struct Opts {
    /// These params can be read also from the configuration file, not just
    /// command-line args or environment variables
    #[clap(flatten)]
    pub shared: SharedOpts,

    /// ZMQ socket name/address for RGB node RPC interface.
    ///
    /// Internal interface for control PRC protocol communications.
    #[clap(
        short = 'R',
        long = "rpc",
        env = "RGB_NODE_RPC_ENDPOINT",
        default_value = RGB_NODE_RPC_ENDPOINT,
        value_hint = ValueHint::FilePath
    )]
    pub rpc_endpoint: ServiceAddr,

    /// ZMQ socket for connecting RGB node message bus.
    #[clap(
        short = 'E',
        long = "storm",
        env = "STORM_NODE_EXT_ENDPOINT",
        default_value = STORM_NODE_EXT_ENDPOINT,
        value_hint = ValueHint::FilePath
    )]
    pub storm_endpoint: ServiceAddr,

    /// Spawn daemons as threads and not processes
    #[clap(short = 't', long = "threaded")]
    pub threaded_daemons: bool,
}

#[cfg(feature = "server")]
impl Opts {
    pub fn process(&mut self) {
        self.shared.process([&mut self.rpc_endpoint, &mut self.storm_endpoint]);
    }
}
