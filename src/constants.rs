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

pub const RGB_BECH32_HRP_ID: &'static str = "rgb";
pub const RGB_BECH32_HRP_GENESIS: &'static str = "rgb:g";
pub const RGB_BECH32_HRP_TRANSITION: &'static str = "rgb:t";
pub const RGB_BECH32_HRP_CONSIGNMENT: &'static str = "rgb:c";
pub const RGB_BECH32_HRP_RGB20ASSET: &'static str = "rgb20a";
pub const RGB_BECH32_HRP_RGB20INVOICE: &'static str = "rgb20i";

pub const RGB_DATA_DIR: &'static str = "/var/lib/rgb";
pub const RGB_BIN_DIR: &'static str = "/usr/local/bin";
pub const RGB_CONTRACTS: &'static str = "fungible";
pub const RGB_NETWORK: &'static str = "testnet";

pub const STASHD_STASH: &'static str = "{data_dir}/{network}/stash/{id}/";
pub const STASHD_INDEX: &'static str = "{data_dir}/{network}/index/";
pub const STASHD_P2P_ENDPOINT: &'static str = "lnp://{node_id}@0.0.0.0:13000";
pub const STASHD_RPC_ENDPOINT: &'static str = "ipc:{data_dir}/{network}/stashd.rpc";
pub const STASHD_PUB_ENDPOINT: &'static str = "ipc:{data_dir}/{network}/stashd.pub";

pub const FUNGIBLED_CACHE: &'static str = "{data_dir}/{network}/cache/fungible";
pub const FUNGIBLED_RPC_ENDPOINT: &'static str = "ipc:{data_dir}/{network}/fungibled.rpc";
pub const FUNGIBLED_PUB_ENDPOINT: &'static str = "ipc:{data_dir}/{network}/fungibled.pub";
