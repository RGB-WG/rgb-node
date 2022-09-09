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
use internet2::addr::{NodeAddr, ServiceAddr};
use lnpbp::chain::Chain;
use rgb::schema::TransitionType;
use rgb::{Contract, ContractId, SealEndpoint};
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
    /// Contract management
    #[clap(subcommand)]
    #[display("contract {0}")]
    Contract(ContractCommand),

    /// Information on outpoints
    #[clap(subcommand)]
    #[display("outpoint {0}")]
    Outpoint(OutpointCommand),

    /// Work with state transfers
    #[clap(subcommand)]
    #[display("transfer {0}")]
    Transfer(TransferCommand),
}

/// Command-line OUTPOINT subcommands:
#[derive(Subcommand, Clone, PartialEq, Eq, Debug, Display)]
pub enum OutpointCommand {
    /// Return known outpoint state
    #[display("state ...")]
    State {
        /// Outpoints to return the state for
        #[clap(short, long = "outpoint")]
        outpoints: Vec<OutPoint>,
    },
}

/// Command-line transfer subcommands:
#[derive(Subcommand, Clone, PartialEq, Eq, Debug, Display)]
pub enum TransferCommand {
    /// Build state transfer consignment draft
    #[display("compose {contract_id} ...")]
    Compose {
        /// Transition types to be always included into the state transfer
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

    /// Update PSBT with the information from the state transition.
    ///
    /// Generates blank state transitions for all other contracts affected
    /// by the given PSBT and adds to PSBT information about the state transition.
    ///
    /// Prepares disclosure for other affected contracts and stores it
    /// internally, linked to the PSBT file txid. Once the txid is seen in
    /// blockchain, the node will enclose the disclosure to the stash
    /// automatically, updating the state of all smart contracts affected by the
    /// transfer.
    ///
    /// The provided PSBT inputs/outputs should not be modified after this
    /// operation (such that txid would not change).
    #[display("combine ...")]
    Combine {
        /// Contract id under which the main state transfer is happening.
        contract_id: ContractId,

        /// File containing main state transition for the transfer.
        transition: PathBuf,

        /// Input file containing PSBT of the transfer witness transaction.
        psbt_in: PathBuf,

        /// Output file to save the PSBT updated with state transition(s)
        /// information. If not given, the source PSBT file is overwritten.
        #[clap(short = 'o', long = "out")]
        psbt_out: Option<PathBuf>,

        /// Bitcoin transaction UTXOs which will be spent by the transfer
        #[clap(required = true)]
        outpoints: Vec<OutPoint>,
    },

    /// Finalize and (optionally) send state transfer consignment to beneficiary.
    ///
    /// Finalize consignment with the information from the finalized PSBT file,
    /// and optionally sends final consignment to beneficiary via Storm Bifrost
    /// (LNP Node).
    #[display("finalize ...")]
    Finalize {
        /// Bifrost server to send state transfer to
        #[clap(short, long)]
        send: Option<NodeAddr>,

        /// Beneficiary blinded TXO seal - or witness transaction output numbers
        /// containing allocations for the beneficiary.
        #[clap(short, long = "endseal", required = true)]
        endseals: Vec<SealEndpoint>,

        /// The final PSBT (not modified).
        psbt: PathBuf,

        /// Output file to save the PSBT updated with state transition(s)
        /// information. If not given, the source PSBT file is overwritten.
        #[clap(short = 'o', long = "out")]
        psbt_out: Option<PathBuf>,

        /// State transfer consignment draft file prepared with `compose` command.
        consignment_in: PathBuf,

        /// Output file to save the final consignment. If not given, the source
        /// consignment file is overwritten.
        consignment_out: Option<PathBuf>,
    },

    /// Validate incoming transfer consignment and consume it into the stash.
    #[display("consume ...")]
    Consume {
        /// Consume even if the endpoint witness transaction is not yet mined.
        #[clap(short, long)]
        force: bool,

        /// State transfer consignment send by the payee.
        consignment: PathBuf,
    },
}

/// Command-line constract subcommands:
#[derive(Subcommand, Clone, PartialEq, Eq, Debug, Display)]
pub enum ContractCommand {
    /// List all known contract ids
    #[display("list")]
    List,

    /// Add new contract to the node
    #[display("register ...")]
    Register {
        /// Force importing of valid contract containing non-mined transactions
        #[clap(long)]
        force: bool,

        /// Contract source in Bech32m encoding (starting with `rgbc1...`)
        contract: Contract,
    },

    /// Query contract state
    #[display("state {contract_id}")]
    State {
        /// Contract id to read state
        contract_id: ContractId,
    },

    /// Request contract consignment
    #[display("consignment {contract_id} ...")]
    Consignment {
        /// Transition types to be always included into the consignment.
        #[clap(short = 't', long = "node-type")]
        node_types: Vec<TransitionType>,

        /// Contract id to read source
        contract_id: ContractId,
    },

    /// Embed contract information into PSBT file
    #[display("embed {contract_id}")]
    Embed {
        /// Transition types to be always included into the consignment.
        #[clap(short = 't', long = "node-type")]
        node_types: Vec<TransitionType>,

        /// Contract id to read state
        contract_id: ContractId,

        /// Input file containing PSBT of the transfer witness transaction.
        psbt_in: PathBuf,

        /// Output file to save the PSBT updated with state transition(s)
        /// information. If not given, the source PSBT file is overwritten.
        #[clap(short = 'o', long = "out")]
        psbt_out: Option<PathBuf>,
    },
}
