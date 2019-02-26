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
use rgb::output_entry::OutputEntry;
use rgb::proof::Proof;
use rgb::traits::Verify;
use std::collections::HashMap;

pub struct ListUnspent {}

impl<'a> RGBSubCommand<'a> for ListUnspent {
    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
        let unspent_utxos = rpc_list_unspent(client).unwrap();

        for (outpoint, amount) in unspent_utxos {
            println!("+---------------------------------------------------------------------+");
            println!("|  {} |", outpoint);
            println!("|    Amount: {:12} SAT                                         |", amount);
            println!("+---------------------------------------------------------------------+");

            let proofs = database.get_proofs_for(&outpoint);

            if proofs.len() == 0 {
                println!("|                          ! NO RGB Proofs !                          |");
            } else {
                for p in proofs {
                    // Verify the proofs here

                    let needed_tx = p.get_needed_txs();
                    let mut needed_map = HashMap::new();

                    fetch_transactions(client, &needed_tx, &mut needed_map);

                    let valid = p.verify(&needed_map);

                    if !valid {
                        eprintln!("|                       !! INVALID RGB PROOF !!                       |");
                        continue;
                    }

                    // -------------------------

                    for entry in p.output {
                        if entry.get_vout().is_some() && entry.get_vout().unwrap() == outpoint.vout {
                            println!("|  {}   |", entry.get_asset_id());
                            println!("|    Amount: {:12}                                             |", entry.get_amount());
                        }
                    }
                }
            }

            println!("+---------------------------------------------------------------------+\n");
        }

        Ok(())
    }
}