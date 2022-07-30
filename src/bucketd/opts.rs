// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use clap::Parser;

use crate::opts::Opts as SharedOpts;

/// Command-line arguments
#[derive(Parser)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[clap(author, version, name = "bucketd", about = "RGB node bucket processor")]
pub struct Opts {
    /// These params can be read also from the configuration file, not just
    /// command-line args or environment variables
    #[clap(flatten)]
    pub shared: SharedOpts,

    #[doc(hidden)]
    #[clap(short = 'R', long = "rpc", hide = true)]
    pub rpc_endpoint: Option<String>,

    #[doc(hidden)]
    #[clap(short = 'E', long = "storm", hide = true)]
    pub storm_endpoint: Option<String>,
}

#[cfg(feature = "server")]
impl Opts {
    pub fn process(&mut self) { self.shared.process([]); }
}
