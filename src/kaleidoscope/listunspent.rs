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
		if matches.is_present("json"){
			let mut vec = Vec::new();
			for (outpoint, amount) in unspent_utxos {

				let proofs = database.get_proofs_for(&outpoint);
				let mut vec2 = Vec::new();

				let txidString= format!("\"txid\": \"{}\"", outpoint.txid);
				let voutString= format!("\"vout\": {}", outpoint.vout);
				let amountString = format!("\"amount\": {}", amount);

				vec2.push (txidString);
				vec2.push (voutString);
				vec2.push (amountString);

				if proofs.len() > 0{
					let mut vec3 = Vec::new();

					for p in proofs {
						// Verify the proofs here

						let needed_tx = p.get_needed_txs();
						let mut needed_map = HashMap::new();

						fetch_transactions(client, &needed_tx, &mut needed_map);

						let valid = p.verify(&needed_map);

						if !valid {
							vec3.push ("{\"error\":\"INVALID RGB PROOF\"}".to_string());
							// vec3.push ("{\"error\":\"INVALID RGB PROOF\"}");
							continue;
						}

						let mut vec4 = Vec::new();
						for entry in p.output {
							if entry.get_vout().is_some() && entry.get_vout().unwrap() == outpoint.vout {
								let asset = format!("\"id\":\"{}\"", entry.get_asset_id());
								let amount = format!("\"amount\":\"{}\"", entry.get_amount());
								vec4.push ([asset, amount].join(","));
							}
						}
						let joinArray = ["{",&vec4.join ("},{"), "}"].join("");
						vec3.push (joinArray);
					}
					let proof = vec3.join (",");
					vec2.push (["\"assets\":[", &proof, "]"].join(""));
				}
				let joinStr = vec2.join(",");
				vec.push (["{", &joinStr, "}"].join(""));
			}
			println!("{{\"listunspent\":[{}]}}", vec.join (","));
		}
		else{
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
		}
		Ok(())
	}
}