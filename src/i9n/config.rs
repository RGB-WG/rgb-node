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

use lnpbp::Chain;

use crate::constants::*;
use crate::rgbd::ContractName;

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    pub verbose: u8,
    pub data_dir: String,
    pub electrum_server: String,
    pub stash_rpc_endpoint: String,
    pub contract_endpoints: HashMap<ContractName, String>,
    pub network: Chain,
    pub run_embedded: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: 0,
            data_dir: RGB_DATA_DIR.to_owned(),
            electrum_server: DEFAULT_ELECTRUM_ENDPOINT.to_owned(),
            stash_rpc_endpoint: STASHD_RPC_ENDPOINT.to_owned(),
            contract_endpoints: map! {
                ContractName::Fungible => FUNGIBLED_RPC_ENDPOINT.to_owned()
            },
            network: RGB_NETWORK
                .parse()
                .expect("Error in RGB_NETWORK constant value"),
            run_embedded: true,
        }
    }
}
