use bitcoin::util::hash::Sha256dHash;
use output_entry::OutputEntry;
use contract::Contract;
use bitcoin::OutPoint;
use bitcoin::network::constants::Network;
use bitcoin::Transaction;
use traits::{Verify, NeededTx};
use std::collections::HashMap;
use bitcoin::TxOut;

#[test]
fn output_entry() {
    let asset_id = Sha256dHash::from_hex(&hex::encode([0x42; 32])).unwrap();
    let output_entry = OutputEntry::new(asset_id, 100, None);
    assert_eq!(output_entry.get_asset_id(), asset_id);
    assert_eq!(output_entry.get_amount(), 100);
    assert_eq!(output_entry.get_vout(), None);
    let output_entry = OutputEntry::new(Sha256dHash::default(), 100, Some(7));
    match output_entry.get_vout() {
        Some(x) => assert_eq!(x, 7),
        _ => panic!()
    }
}

#[test]
fn verify() {
    let issuance_utxo = OutPoint {txid: Sha256dHash::default(), vout: 1000};
    let contract = Contract {
        title: String::from("title"),
        issuance_utxo,
        initial_owner_utxo: issuance_utxo,
        network: Network::Testnet,
        total_supply: 12,
    };
    let void_tx = Transaction {version: 1, lock_time: 0, input: vec![], output: vec![]};

    let needed_txs = contract.get_needed_txs();
    assert_eq!(needed_txs.len(), 1);
    // let NeededTx::WhichSpendsOutPoint(outpoint) = needed_txs[0];
    let outpoint = match needed_txs[0] {
        NeededTx::WhichSpendsOutPoint(o) => o,
        _ => panic!(),
    };
    assert_eq!(outpoint, contract.issuance_utxo);

    let mut txs: HashMap<&NeededTx, Transaction> = [
        (&needed_txs[0], void_tx)
    ].iter().cloned().collect();

    assert_eq!(contract.verify(&txs), false);

    let commitment_out: TxOut = TxOut {
        script_pubkey: contract.get_expected_script(),
        value: 7
    };
    println!("test: expected {}", contract.get_expected_script());
    let issuance_tx = Transaction {version: 1, lock_time: 0, input: vec![], output: vec![commitment_out]};

    txs = [
        (&needed_txs[0], issuance_tx)
    ].iter().cloned().collect();

    assert_eq!(contract.verify(&txs), true);
}
