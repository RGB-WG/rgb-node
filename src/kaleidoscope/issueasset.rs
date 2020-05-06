use std::collections::HashMap;

use bitcoin::network::constants::Network;
use bitcoin::OutPoint;
use bitcoin::Privkey;
use bitcoin::util::hash::Sha256dHash;
use clap::ArgMatches;
use jsonrpc;
use jsonrpc::client::Client;
use rgb::contract::Contract;
use rgb::output_entry::OutputEntry;
use rgb::proof::Proof;
use secp256k1::PublicKey;
use secp256k1::Secp256k1;

use chain::wallet::*;
use database::Database;
use kaleidoscope::{Config, RGBSubCommand};
use lib::tx_builder::{build_issuance_tx, raw_tx_commit_to, raw_tx_commit_to_p2c};

pub struct IssueAsset {}

impl<'a> RGBSubCommand<'a> for IssueAsset {
    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
        let unspent_utxos = rpc_list_unspent(client).unwrap();
        let mut unspent_utxos_outpoints: Vec<&OutPoint> = unspent_utxos.keys().collect();
        // filter out the UTXOs with proof attached to them
        let unspent_utxos_outpoints: Vec<&&OutPoint> = unspent_utxos_outpoints
            .iter()
            .filter(|outpoint| database.get_proofs_for(*outpoint).len() == 0)
            .collect();

        const FEE: u64 = 3000;
		const necessary_number_utxos: usize = 2;

        if unspent_utxos_outpoints.len() < necessary_number_utxos {
            eprintln!("At least {} UTXOs are needed!", necessary_number_utxos);
            return Err(jsonrpc::Error::NoErrorOrResult);
        }

        let issuance_utxo: OutPoint = match matches.value_of("issuance_utxo") {
            Some(utxo) => {
                let parts: Vec<&str> = utxo.split(":").collect();

                OutPoint {
                    txid: Sha256dHash::from_hex(parts[0]).unwrap(),
                    vout: parts[1].parse().unwrap(),
                }
            }
            None => *unspent_utxos_outpoints[0].clone()
        };

        let initial_owner_utxo: OutPoint = match matches.value_of("initial_owner_utxo") {
            Some(utxo) => {
                let parts: Vec<&str> = utxo.split(":").collect();

                OutPoint {
                    txid: Sha256dHash::from_hex(parts[0]).unwrap(),
                    vout: parts[1].parse().unwrap(),
                }
            }
            None => *unspent_utxos_outpoints[1].clone()
        };

        let network = match matches.value_of("network").unwrap() {
            "mainnet" => Network::Bitcoin,
            "testnet" => Network::Testnet,
            "regtest" => Network::Regtest,
            _ => panic!("Invalid network")
        };

        // -------------------------------------

        let s = Secp256k1::new();

        let burn_address = rpc_getnewaddress(client).unwrap();

        let mut contract = Contract {
            title: matches.value_of("title").unwrap().to_string(),
            total_supply: matches.value_of("total_supply").unwrap().parse().unwrap(),
            network,
            issuance_utxo,
            initial_owner_utxo,
            original_commitment_pk: None
        };

        let change_address = rpc_getnewaddress(client).unwrap();
        let change_privkey = rpc_dumpprivkey(client, &change_address).unwrap();
        let change_pubkey = PublicKey::from_secret_key(&s, &change_privkey.key);
        let change_amount = unspent_utxos.get(&contract.issuance_utxo).unwrap() - FEE;

        let mut commit_tx_outputs = HashMap::new();

        let (issuance_tx, tweak_factor) = build_issuance_tx(&mut contract, &change_pubkey, change_amount, &commit_tx_outputs);
        let issuance_tx = rpc_sign_transaction(client, &issuance_tx).unwrap();

        println!("Asset ID: {}", contract.get_asset_id());
        println!("Spending the issuance_utxo {} in {}", contract.issuance_utxo, issuance_tx.txid());

        let mut tweaked_sk = change_privkey.key.clone();
        tweaked_sk.add_assign(&s, &tweak_factor.as_inner());
        let tweaked_privkey = Privkey::from_secret_key(tweaked_sk, true, change_privkey.network);

        rpc_importprivkey(client, &tweaked_privkey);

        println!("Importing the tweaked private key for the contract commitment...");

        // -------------------------------------

        let mut root_proof = Proof::new(
            vec![contract.initial_owner_utxo.clone()],
            vec![],
            vec![OutputEntry::new(contract.get_asset_id(), contract.total_supply, Some(0))],
            Some(&contract),
            None);

        let root_proof_change_address = rpc_getnewaddress(client).unwrap();
        let root_proof_change_privkey = rpc_dumpprivkey(client, &root_proof_change_address).unwrap();
        let root_proof_change_pubkey = PublicKey::from_secret_key(&s, &root_proof_change_privkey.key);
        let root_proof_change_amount = unspent_utxos.get(&contract.initial_owner_utxo).unwrap() - FEE;

        let mut proof_commit_tx_outputs = HashMap::new();

        let (root_proof_tx, root_proof_tweak_factor) = raw_tx_commit_to_p2c(
            &mut root_proof,
            vec![contract.initial_owner_utxo.clone()],
            &root_proof_change_pubkey,
            root_proof_change_amount,
            &proof_commit_tx_outputs,
        );
        let root_proof_tx = rpc_sign_transaction(client, &root_proof_tx).unwrap();

        println!("Spending the initial_owner_utxo {} in {}", contract.initial_owner_utxo, root_proof_tx.txid());

        let mut tweaked_sk = root_proof_change_privkey.key.clone();
        tweaked_sk.add_assign(&s, &root_proof_tweak_factor.as_inner());
        let tweaked_privkey = Privkey::from_secret_key(tweaked_sk, true, root_proof_change_privkey.network);

        rpc_importprivkey(client, &tweaked_privkey);

        println!("Importing the tweaked private key for the root proof commitment...");

        database.save_proof(&root_proof, &root_proof_tx.txid());

        rpc_broadcast(client, &issuance_tx)?;
        rpc_broadcast(client, &root_proof_tx)?;

        Ok(())
    }
}