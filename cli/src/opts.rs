// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::path::PathBuf;

use bitcoin::OutPoint;
use internet2::addr::ServiceAddr;
use lnpbp::chain::Chain;
use rgb::schema::TransitionType;
use rgb::{Contract, ContractId};
use rgb_rpc::RGB_NODE_RPC_ENDPOINT;

/// Command-line tool for working with RGB node
#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[clap(name = "rgb-cli", bin_name = "rgb-cli", author, version)]
pub struct Opts {
    /// ZMQ socket for connecting daemon RPC interface.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    ///
    /// Defaults to `0.0.0.0:61961`.
    #[clap(
        short = 'R',
        long = "rpc",
        global = true,
        default_value = RGB_NODE_RPC_ENDPOINT,
        env = "RGB_NODE_RPC_ENDPOINT"
    )]
    pub connect: ServiceAddr,

    /// Blockchain to use
    #[clap(
        short = 'n',
        long,
        global = true,
        alias = "network",
        default_value = "signet",
        env = "RGB_NETWORK"
    )]
    pub chain: Chain,

    /// Set verbosity level.
    ///
    /// Can be used multiple times to increase verbosity.
    #[clap(short, long, global = true, parse(from_occurrences))]
    pub verbose: u8,

    /// Command to execute
    #[clap(subcommand)]
    pub command: Command,
}

/// Command-line commands:
#[derive(Subcommand, Clone, PartialEq, Eq, Debug, Display)]
pub enum Command {
    /// Add new contract to the node
    #[display("register ...")]
    Register {
        /// Force importing of valid contract containing non-mined transactions
        #[clap(long)]
        force: bool,

        /// Contract source in Bech32m encoding (starting with `rgbc1...`)
        contract: Contract,
    },

    /// List all known contract ids
    #[display("contracts")]
    Contracts,

    /// Query contract state
    #[display("state {contract_id}")]
    State {
        /// Contract id to read state
        contract_id: ContractId,
    },

    /// Request contract source
    #[display("contract {contract_id} ...")]
    Contract {
        #[clap(short = 't', long = "node-type")]
        node_types: Vec<TransitionType>,

        /// Contract id to read source
        contract_id: ContractId,
    },

    /// Request contract source
    #[display("consign {contract_id} ...")]
    Consign {
        #[clap(short = 't', long = "node-type")]
        node_types: Vec<TransitionType>,

        /// Contract id to read source
        contract_id: ContractId,

        /// Bitcoin transaction UTXOs which will be spent by the transfer
        #[clap(required = true)]
        outpoints: Vec<OutPoint>,

        /// Output file to save consignment prototype to
        output: PathBuf,
    },
}
