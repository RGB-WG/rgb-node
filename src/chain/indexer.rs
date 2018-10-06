extern crate bitcoin;
extern crate strason;

use bitcoin::Block;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::network::serialize;
use bitcoin::Transaction;
use bitcoin::util::hash::Sha256dHash;
use jsonrpc;
use jsonrpc::client::Client;
use rgb::traits::NeededTx;
use rgb::traits::NeededTx::FromTXID;
use rgb::traits::NeededTx::WhichSpendsOutPoint;
use rgb::utils::hex_to_bytes;
use self::strason::Json;
use std::collections::HashMap;


fn rpc_get_transaction(client: &mut Client, txid: &Sha256dHash) -> Result<Transaction, jsonrpc::Error> {
    let request = client
        .build_request("getrawtransaction".to_string(), vec![Json::from_serialize(txid.to_string()).unwrap()]);

    client.send_request(&request).and_then(|res| {
        let tx: Result<Transaction, bitcoin::network::serialize::Error> = bitcoin::network::serialize::deserialize(&mut hex_to_bytes(String::from(res.result.unwrap().string().unwrap())));

        Ok(tx.unwrap())
    })
}

// TODO: as suggested by @sjors, we could try to replace this with a "getrawtransaction <txid> true"
fn rpc_find_where_spent(client: &mut Client, outpoint: &OutPoint) -> Result<Transaction, jsonrpc::Error> {
    let request = client
        .build_request("getbestblockhash".to_string(), vec![]);

    let mut block_hash = String::from("");

    client.send_request(&request).and_then(|res| {
        block_hash = String::from(res.result.unwrap().string().unwrap());

        Ok(())
    });

    let mut tx: Option<Transaction> = None;

    while tx.is_none() {
        let request = client
            .build_request("getblock".to_string(), vec![Json::from_serialize(block_hash.to_string()).unwrap(), Json::from_serialize(0).unwrap()]);

        client.send_request(&request).and_then(|res| {
            let block: Result<Block, bitcoin::network::serialize::Error> = bitcoin::network::serialize::deserialize(&mut hex_to_bytes(String::from(res.result.unwrap().string().unwrap())));
            let block = block.unwrap();

            for this_tx in &block.txdata {
                for vin in &this_tx.input {
                    if vin.previous_output == *outpoint {
                        tx = Some(this_tx.clone());

                        break;
                    }
                }

                if tx.is_some() {
                    break;
                }
            }

            block_hash = block.header.prev_blockhash.to_string();

            Ok(())
        });
    }

    Ok(tx.unwrap())
}

pub fn fetch_transactions<'a>(client: &mut Client, needed_txs: &'a Vec<NeededTx>, map: &mut HashMap<&'a NeededTx, Transaction>) {
    for need in needed_txs {
        let tx = match need {
            FromTXID(txid) => rpc_get_transaction(client, txid),
            WhichSpendsOutPoint(outpoint) => rpc_find_where_spent(client, outpoint)
        };

        if let Ok(tx_val) = tx {
            map.insert(need, tx_val.clone());
        }
    }
}