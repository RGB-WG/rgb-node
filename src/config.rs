// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::fs;
use std::path::PathBuf;

use internet2::addr::ServiceAddr;
use rgb_rpc::RGB_NODE_RPC_ENDPOINT;
use storm_app::STORM_NODE_APP_ENDPOINT;

#[cfg(feature = "server")]
use crate::opts::Opts;
use crate::{containerd, rgbd};

/// Final configuration resulting from data contained in config file environment
/// variables and command-line options. For security reasons node key is kept
/// separately.
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    /// ZMQ socket for RPC API.
    pub rpc_endpoint: ServiceAddr,

    /// ZMQ socket for RPC API.
    pub ctl_endpoint: ServiceAddr,

    /// ZMQ socket for Storm node service bus.
    pub storm_endpoint: ServiceAddr,

    /// ZMQ socket for Store service RPC.
    pub store_endpoint: ServiceAddr,

    /// Data location
    pub data_dir: PathBuf,

    /// Verbosity level
    pub verbose: u8,
}

#[cfg(feature = "server")]
impl Config {
    pub fn process(&mut self) {
        self.data_dir =
            PathBuf::from(shellexpand::tilde(&self.data_dir.display().to_string()).to_string());

        let me = self.clone();
        let mut data_dir = self.data_dir.to_string_lossy().into_owned();
        self.process_dir(&mut data_dir);
        self.data_dir = PathBuf::from(data_dir);

        fs::create_dir_all(&self.data_dir).expect("Unable to access data directory");

        for dir in vec![
            &mut self.rpc_endpoint,
            &mut self.ctl_endpoint,
            &mut self.storm_endpoint,
            &mut self.store_endpoint,
        ] {
            if let ServiceAddr::Ipc(ref mut path) = dir {
                me.process_dir(path);
            }
        }
    }

    pub fn process_dir(&self, path: &mut String) {
        *path = path.replace("{data_dir}", &self.data_dir.to_string_lossy());
        *path = shellexpand::tilde(path).to_string();
    }
}

#[cfg(feature = "server")]
impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Config {
            data_dir: opts.data_dir,
            rpc_endpoint: RGB_NODE_RPC_ENDPOINT.parse().expect("error in constant value"),
            ctl_endpoint: opts.ctl_endpoint,
            storm_endpoint: STORM_NODE_APP_ENDPOINT.parse().expect("error in constant value"),
            store_endpoint: opts.store_endpoint,
            verbose: opts.verbose,
        }
    }
}

impl From<rgbd::Opts> for Config {
    fn from(opts: rgbd::Opts) -> Config {
        let mut config = Config::from(opts.shared);
        config.set_storm_endpoint(opts.storm_endpoint);
        config.set_rpc_endpoint(opts.rpc_endpoint);
        config
    }
}

impl From<containerd::Opts> for Config {
    fn from(opts: containerd::Opts) -> Config { Config::from(opts.shared) }
}

impl Config {
    pub fn set_rpc_endpoint(&mut self, endpoint: ServiceAddr) { self.rpc_endpoint = endpoint; }
    pub fn set_storm_endpoint(&mut self, endpoint: ServiceAddr) { self.storm_endpoint = endpoint; }
}
