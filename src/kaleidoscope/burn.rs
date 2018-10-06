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
use kaleidoscope::sendtoaddress::send_to_address;
use rgb::contract::Contract;
use rgb::proof::OutputEntry;
use rgb::proof::Proof;
use rgb::traits::Verify;
use std::collections::HashMap;

pub struct Burn {}

impl<'a> RGBSubCommand<'a> for Burn {
    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
        let asset_id = Sha256dHash::from_hex(matches.value_of("asset_id").unwrap()).unwrap();
        let amount: u32 = matches.value_of("amount").unwrap().parse().unwrap();

        let unspent_utxos = rpc_list_unspent(client).unwrap();
        let mut burn_address = None;

        // TODO: save contracts in the database to avoid looking for them like that
        'outer: for (outpoint, amount) in unspent_utxos {
            let proofs = database.get_proofs_for(&outpoint);

            for p in proofs {
                for entry in &p.output {
                    if entry.get_vout() == outpoint.vout {
                        if entry.get_asset_id() == asset_id {
                            burn_address = Some(p.get_contract_for(asset_id.clone()).unwrap().burn_address);
                            break 'outer;
                        }
                    }
                }
            }
        }

        if !burn_address.is_some() {
            println!("Contract not found for {}", asset_id);
            return Err(jsonrpc::Error::NoErrorOrResult);
        }

        send_to_address(burn_address.unwrap(), config.default_server.as_str(), asset_id, amount, config, database, client)
    }
}