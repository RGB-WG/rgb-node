use bifrost::upload_proofs;
use bitcoin::Address;
use bitcoin::network::constants::Network;
use bitcoin::network::serialize::BitcoinHash;
use bitcoin::OutPoint;
use bitcoin::Privkey;
use bitcoin::util::hash::Sha256dHash;
use chain::tx_builder::BitcoinRgbOutPoints;
use chain::tx_builder::spend_proofs;
use chain::wallet::*;
use clap::ArgMatches;
use database::Database;
use jsonrpc;
use jsonrpc::client::Client;
use kaleidoscope::{Config, RGBSubCommand};
use rgb::contract::Contract;
use rgb::proof::OutputEntry;
use rgb::proof::Proof;
use secp256k1::PublicKey;
use secp256k1::Secp256k1;
use std::cmp;
use std::collections::HashMap;
use std::str::FromStr;

pub struct SendToAddress {}

pub fn send_to_address(btc_address: Address, server: &str, asset_id: Sha256dHash, amount: u32, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
    const FEE: u64 = 2000;

    let s = Secp256k1::new();

    let change_address = rpc_getnewaddress(client).unwrap();
    let change_privkey = rpc_dumpprivkey(client, &change_address).unwrap();
    let change_pubkey = PublicKey::from_secret_key(&s, &change_privkey.key);

    // -----------------

    let unspent_utxos = rpc_list_unspent(client).unwrap();

    let mut chosen_outpoints = Vec::new();
    let mut chosen_proofs = Vec::new();
    let mut total_btc_amount: u64 = 0;
    let mut total_asset_amount: u32 = 0;
    let mut to_self: HashMap<Sha256dHash, u32> = HashMap::new();

    let mut used_proofs = HashMap::new();

    for (outpoint, btc_amount) in &unspent_utxos {
        let proofs = database.get_proofs_for(&outpoint);
        let mut used = false;

        // While theoretically there could be more proofs for the same outpoint,
        // in this basic version the only way to bind some tokens to a UTXO
        // is by actually creating it. Thus, since the same output cannot be created
        // twice, we will always have at most one proof.

        if proofs.len() == 0 {
            continue;
        }

        let p = &proofs[0];

        for entry in &p.output {
            if entry.get_vout() == outpoint.vout { // entry for us
                used = true;

                if entry.get_asset_id() != asset_id { // full back to self, different asset
                    let aggregator = to_self.entry(entry.get_asset_id()).or_insert(0);
                    *aggregator += entry.get_amount();
                } else {
                    let use_from_this = cmp::min(
                        amount - total_asset_amount, // remaining part
                        entry.get_amount(), // or all of it
                    );

                    total_asset_amount += use_from_this;

                    if use_from_this < entry.get_amount() { // partial back to self
                        let aggregator = to_self.entry(entry.get_asset_id()).or_insert(0);
                        *aggregator += entry.get_amount() - use_from_this;
                    }
                }
            }
        }

        if used {
            total_btc_amount += btc_amount; // add the btc amount
            chosen_outpoints.push(outpoint.clone()); // set as input for the tx

            // Even though each output will only have (at most) one proof, it's still possible that
            // multiple outputs share the same proof. This is why we need to keep track of the ones
            // already spent.

            if !used_proofs.get(&p.bitcoin_hash()).is_some() { // hasn't been used
                chosen_proofs.push(p.clone()); // spend the proof
                used_proofs.insert(p.bitcoin_hash(), true); // mark as used
            }
        }

        if total_asset_amount == amount { // we are done here
            break;
        }
    }

    if total_asset_amount < amount {
        println!("Insufficient funds! {} < {}", total_asset_amount, amount);
        return Err(jsonrpc::Error::NoErrorOrResult);
    }

    // --------------------------------------

    let mut rgb_outputs = Vec::new();

    total_btc_amount -= FEE;
    let payment_amount = total_btc_amount / 2;

    // payment
    let mut payment_map = HashMap::new();
    payment_map.insert(asset_id.clone(), amount);
    rgb_outputs.push(BitcoinRgbOutPoints::new(btc_address.clone(), payment_amount, payment_map));

    let (final_p, final_tx, tweak_factor) = spend_proofs(&chosen_proofs, &chosen_outpoints, &change_pubkey, total_btc_amount - payment_amount, &to_self.clone(), &rgb_outputs);

    // Pay to contract
    let mut tweaked_sk = change_privkey.key.clone();
    tweaked_sk.add_assign(&s, &tweak_factor.as_inner());
    let tweaked_privkey = Privkey::from_secret_key(tweaked_sk, true, change_privkey.network);

    let new_change_address = Address::p2pkh(&PublicKey::from_secret_key(&s, &tweaked_privkey.key), change_privkey.network);

    rpc_importprivkey(client, &tweaked_privkey);

    println!("Importing the tweaked private key for the proof commitment...");

    // ---------------------------------------

    let final_tx = rpc_sign_transaction(client, &final_tx).unwrap();

    println!("Created a new TX with the following outputs:");

    // 0 = change
    for (to_self_asset, to_self_amount) in &to_self {
        println!("\t[CHANGE] {} of {} to {}", to_self_amount, to_self_asset, new_change_address.clone());
    }
    println!("\t[CHANGE] {} SAT to {}", total_btc_amount - payment_amount, new_change_address.clone());

    // 1 = payment
    println!("\t         {} of {} to {}", amount, asset_id, btc_address.clone());
    println!("\t         {} SAT to {}", payment_amount, btc_address.clone());

    println!("TXID: {}", final_tx.txid());

    // ----------------------------------------

    //println!("{:#?}", final_p);

    // upload to server
    upload_proofs(&String::from(server), &final_p, &final_tx.txid()).unwrap();
    println!("Proof uploaded to {}", server);

    database.save_proof(&final_p, &final_tx.txid());
    rpc_broadcast(client, &final_tx);

    Ok(())
}

impl<'a> RGBSubCommand<'a> for SendToAddress {
    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
        let address_parts: Vec<&str> = matches.value_of("address").unwrap().split("@").collect();
        let btc_address = Address::from_str(address_parts[0]).unwrap();
        let server = address_parts[1];

        let asset_id = Sha256dHash::from_hex(matches.value_of("asset_id").unwrap()).unwrap();
        let amount: u32 = matches.value_of("amount").unwrap().parse().unwrap();

        send_to_address(btc_address, server, asset_id, amount, config, database, client)
    }
}