use bitcoin::network::constants::Network;
use bitcoin::OutPoint;
use bitcoin::util::hash::Sha256dHash;
use chain::indexer::fetch_transactions;
use chain::tx_builder::{build_issuance_tx, raw_tx_commit_to};
use chain::wallet::*;
use clap::ArgMatches;
use database::Database;
use jsonrpc;
use jsonrpc::client::Client;
use kaleidoscope::{Config, RGBSubCommand};
use rgb::contract::Contract;
use rgb::proof::OutputEntry;
use rgb::proof::Proof;
use rgb::traits::Verify;
use std::collections::HashMap;

pub struct GetNewAddress {}

impl<'a> RGBSubCommand<'a> for GetNewAddress {
    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
        let server = matches.value_of("server").unwrap_or(config.default_server.as_str());

        let address = rpc_getnewaddress(client).unwrap();

        println!("{}@{}", address, server);

        Ok(())
    }
}