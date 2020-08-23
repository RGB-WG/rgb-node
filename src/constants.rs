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

//! Shared constants, including configuration parameters etc

pub const RGB20_BECH32_HRP_INVOICE: &'static str = "rgb20:";

pub const RGB_DATA_DIR: &'static str = "/var/lib/rgb";
pub const RGB_BIN_DIR: &'static str = "/usr/local/bin";
pub const RGB_CONTRACTS: &'static str = "fungible";
pub const RGB_NETWORK: &'static str = "testnet";

pub const STASHD_STASH: &'static str = "{data_dir}/{network}/stash/{id}/";
pub const STASHD_INDEX: &'static str = "{data_dir}/{network}/stash/{id}/index.dat";
pub const STASHD_P2P_ENDPOINT: &'static str = "lnp://{node_id}@0.0.0.0:13000";
pub const STASHD_RPC_ENDPOINT: &'static str = "ipc:{data_dir}/{network}/stashd.rpc";
pub const STASHD_PUB_ENDPOINT: &'static str = "ipc:{data_dir}/{network}/stashd.pub";

pub const FUNGIBLED_CACHE: &'static str = "{data_dir}/{network}/cache/fungible";
pub const FUNGIBLED_RPC_ENDPOINT: &'static str = "ipc:{data_dir}/{network}/fungibled.rpc";
pub const FUNGIBLED_PUB_ENDPOINT: &'static str = "ipc:{data_dir}/{network}/fungibled.pub";
