extern crate bitcoin;
extern crate strason;

use bitcoin::Address;
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
use rgb::utils::{bytes_to_hex, hex_to_bytes};
use self::strason::Json;
use std::collections::HashMap;
use std::str::FromStr;

pub fn rpc_sign_transaction(client: &mut Client, tx: &Transaction) -> Result<Transaction, jsonrpc::Error> {
    use bitcoin::network::serialize::serialize;
    let encoded = serialize(tx).unwrap();

    let str_encoded = bytes_to_hex(&encoded);

    let request = client
        .build_request("signrawtransaction".to_string(), vec![Json::from_serialize(str_encoded).unwrap()]);

    client.send_request(&request).and_then(|res| {
        if res.error.is_some() {
            println!("Sign error: {:?}", res.error.unwrap());
            return Err(jsonrpc::Error::NoErrorOrResult);
        }

        let raw_tx_string = String::from(res.result.unwrap().get("hex").unwrap().string().unwrap());
        let signed_tx: Result<Transaction, bitcoin::network::serialize::Error> = bitcoin::network::serialize::deserialize(&mut hex_to_bytes(raw_tx_string));

        Ok(signed_tx.unwrap())
    })
}

pub fn rpc_list_unspent(client: &mut Client) -> Result<HashMap<OutPoint, u64>, jsonrpc::Error> {
    let request = client
        .build_request("listunspent".to_string(), vec![]);

    client.send_request(&request).and_then(|res| {
        if res.error.is_some() {
            println!("List unspent error: {:?}", res.error.unwrap());
            return Err(jsonrpc::Error::NoErrorOrResult);
        }

        let mut ans: HashMap<OutPoint, u64> = HashMap::new();
        let result = res.result.unwrap();

        for i in 0..result.len() {
            if !result.array().unwrap()[i].get("spendable").unwrap().bool().unwrap() {
                continue;
            }
            let op = OutPoint {
                txid: Sha256dHash::from_hex(result.array().unwrap()[i].get("txid").unwrap().string().unwrap()).unwrap(),
                vout: result.array().unwrap()[i].get("vout").unwrap().num().unwrap().parse().unwrap(),
            };

            let amount = f64::from_str(result.array().unwrap()[i].get("amount").unwrap().num().unwrap()).unwrap() * 1e8;

            ans.insert(op, amount as u64);
        }

        Ok(ans)
    })
}

pub fn rpc_broadcast(client: &mut Client, tx: &Transaction) -> Result<(), jsonrpc::Error> {
    use bitcoin::network::serialize::serialize;
    let encoded = serialize(tx).unwrap();

    let str_encoded = bytes_to_hex(&encoded);

    let request = client
        .build_request("sendrawtransaction".to_string(), vec![Json::from_serialize(str_encoded).unwrap()]);

    client.send_request(&request).and_then(|res| {
        if res.error.is_some() {
            println!("Broadcast error: {:?}", res.error.unwrap());
            return Err(jsonrpc::Error::NoErrorOrResult);
        }

        Ok(())
    })
}

pub fn rpc_getnewaddress(client: &mut Client) -> Result<Address, jsonrpc::Error> {
    let request = client
        .build_request("getnewaddress".to_string(), vec![]);

    client.send_request(&request).and_then(|res| {
        if res.error.is_some() {
            println!("GetNewAddress error: {:?}", res.error.unwrap());
            return Err(jsonrpc::Error::NoErrorOrResult);
        }

        Ok(Address::from_str(res.result.unwrap().string().unwrap()).unwrap())
    })
}