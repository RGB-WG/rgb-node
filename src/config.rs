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

/// Final configuration resulting from data contained in config file environment
/// variables and command-line options. For security reasons node key is kept
/// separately.
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    /// ZMQ socket for RPC API.
    pub rpc_endpoint: ServiceAddr,

    /// ZMQ socket for Storm node service bus.
    pub storm_endpoint: ServiceAddr,

    /// ZMQ socket for Store service RPC.
    pub store_endpoint: ServiceAddr,

    /// Data location
    pub data_dir: PathBuf,

    /// Verbosity level
    pub verbose: u8,
}

impl Config {
    pub fn process(&mut self) {
        self.data_dir =
            PathBuf::from(shellexpand::tilde(&self.data_dir.display().to_string()).to_string());

        let me = self.clone();
        let mut data_dir = self.data_dir.to_string_lossy().into_owned();
        self.process_dir(&mut data_dir);
        self.data_dir = PathBuf::from(data_dir);

        fs::create_dir_all(&self.data_dir).expect("Unable to access data directory");

        for dir in vec![&mut self.rpc_endpoint] {
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
