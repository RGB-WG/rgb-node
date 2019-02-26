use bifrost::get_proofs_for;
use bifrost::upload_proofs;
use bitcoin::network::constants::Network;
use bitcoin::network::serialize::BitcoinHash;
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

pub struct Sync {}

impl<'a> RGBSubCommand<'a> for Sync {
    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
        let server = String::from(matches.value_of("server").unwrap_or(config.default_server.as_str()));
        let unspent_utxos = rpc_list_unspent(client).unwrap();

        for (outpoint, amount) in unspent_utxos {
            for p in database.get_proofs_for(&outpoint) { // first upload
                println!(" --> Uploaded proof {}", p.bitcoin_hash());
                upload_proofs(&server, &p, &outpoint.txid);
            }

            // ---------------------------- // TODO do not re-download proofs we already have

            // then download
            let downloaded = get_proofs_for(&server, &outpoint).unwrap();
            for p in downloaded {
                println!(" <-- Downloaded proof for {}", outpoint);
                print!(" *** Starting the verification process:");

                let needed_tx = p.get_needed_txs();
                let mut needed_map = HashMap::new();

                fetch_transactions(client, &needed_tx, &mut needed_map);

                let valid = p.verify(&needed_map);

                if !valid {
                    println!(" [INVALID]");
                    continue;
                }

                println!(" [VALID]");
                database.save_proof(&p, &outpoint.txid);
            }
        }

        Ok(())
    }
}