use std::collections::HashMap;

use bitcoin::network::constants::Network;
use bitcoin::OutPoint;
use bitcoin::util::hash::Sha256dHash;
use clap::ArgMatches;
use jsonrpc;
use jsonrpc::client::Client;
use rgb::contract::Contract;
use rgb::output_entry::OutputEntry;
use rgb::proof::Proof;
use rgb::traits::Verify;

use chain::indexer::fetch_transactions;
use chain::wallet::*;
use database::Database;
use kaleidoscope::{Config, RGBSubCommand};
use lib::tx_builder::{build_issuance_tx, raw_tx_commit_to};

pub struct GetNewAddress {}

impl<'a> RGBSubCommand<'a> for GetNewAddress {
    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
        let server = matches.value_of("server").unwrap_or(config.rgb_server.as_str());

        let address = rpc_getnewaddress(client).unwrap();

        println!("{}@{}", address, server);

        Ok(())
    }
}