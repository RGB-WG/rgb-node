// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::HashMap;

use lnpbp::bp;
use lnpbp::lnp::transport::zmq::SocketLocator;

use crate::constants::*;
use crate::rgbd::ContractName;

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Config {
    pub stash_endpoint: SocketLocator,
    pub contract_endpoints: HashMap<ContractName, SocketLocator>,
    pub network: bp::Network,
    pub threaded: bool,
    pub data_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stash_endpoint: STASHD_RPC_ENDPOINT
                .parse()
                .expect("Error in STASHD_RPC_ENDPOINT constant value"),
            contract_endpoints: map! {
                ContractName::Fungible
                    => FUNGIBLED_RPC_ENDPOINT
                        .parse()
                        .expect("Error in FUNGIBLED_RPC_ENDPOINT constant value")
            },
            network: RGB_NETWORK
                .parse()
                .expect("Error in RGB_NETWORK constant value"),
            threaded: true,
            data_dir: RGB_DATA_DIR.to_string(),
        }
    }
}
