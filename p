diff --git a/src/bifrost.rs b/src/bifrost.rs
index a3f11af..35f69c2 100644
--- a/src/bifrost.rs
+++ b/src/bifrost.rs
@@ -10,7 +10,10 @@ use std::io::Read;
 
 pub fn upload_proofs(server: &String, proof: &Proof, txid: &Sha256dHash) -> Result<(), Error> {
     for out in &proof.output {
-        let outpoint_str = txid.be_hex_string() + ":" + out.get_vout().to_string().as_str();
+        let outpoint_str = match out.get_vout() {
+            Some(vout) => txid.be_hex_string() + ":" + vout.to_string().as_str(),
+            None => txid.be_hex_string() + ":BURN"
+        };
         let url = format!("http://{}/{}", server, outpoint_str);
 
         let client = Client::new();
diff --git a/src/chain/tx_builder.rs b/src/chain/tx_builder.rs
index 9d357c1..b6f5ad9 100644
--- a/src/chain/tx_builder.rs
+++ b/src/chain/tx_builder.rs
@@ -4,7 +4,7 @@ use bitcoin::blockdata::script::Script;
 use bitcoin::OutPoint;
 use bitcoin::util::hash::Sha256dHash;
 use rgb::contract::Contract;
-use rgb::proof::OutputEntry;
+use rgb::output_entry::OutputEntry;
 use rgb::proof::Proof;
 use rgb::traits::Verify;
 use std::collections::HashMap;
@@ -45,13 +45,13 @@ pub fn build_issuance_tx(contract: &Contract, outputs: &HashMap<Address, u64>) -
 
 #[derive(Clone, Debug)]
 pub struct BitcoinRgbOutPoints {
-    pub bitcoin_address: Address,
+    pub bitcoin_address: Option<Address>,
     pub bitcoin_amount: u64,
     pub rgb_outputs: HashMap<Sha256dHash, u32>,
 }
 
 impl BitcoinRgbOutPoints {
-    pub fn new(bitcoin_address: Address, bitcoin_amount: u64, rgb_outputs: HashMap<Sha256dHash, u32>) -> BitcoinRgbOutPoints {
+    pub fn new(bitcoin_address: Option<Address>, bitcoin_amount: u64, rgb_outputs: HashMap<Sha256dHash, u32>) -> BitcoinRgbOutPoints {
         BitcoinRgbOutPoints {
             bitcoin_address,
             bitcoin_amount,
@@ -96,19 +96,27 @@ pub fn spend_proofs(input_proofs: &Vec<Proof>, bitcoin_inputs: &Vec<OutPoint>, o
     let mut tx_out_index = 0;
 
     for output_item in outputs {
-        let this_tx_out = TxOut {
-            value: output_item.bitcoin_amount,
-            script_pubkey: output_item.bitcoin_address.script_pubkey(),
-        };
-
-        tx_outs.push(this_tx_out);
-
-        // Add the RGB outpoints
-        for (asset_id, amount) in &output_item.rgb_outputs {
-            proof.output.push(OutputEntry::new(asset_id.clone(), amount.clone(), tx_out_index));
+        match output_item.bitcoin_address {
+            Some(ref addr) => {
+                let this_tx_out = TxOut {
+                    value: output_item.bitcoin_amount,
+                    script_pubkey: addr.script_pubkey(),
+                };
+
+                tx_outs.push(this_tx_out);
+
+                for (asset_id, amount) in &output_item.rgb_outputs {
+                    proof.output.push(OutputEntry::new(asset_id.clone(), amount.clone(), Some(tx_out_index)));
+                }
+
+                tx_out_index += 1;
+            },
+            None => {
+                for (asset_id, amount) in &output_item.rgb_outputs {
+                    proof.output.push(OutputEntry::new(asset_id.clone(), amount.clone(), None));
+                }
+            }
         }
-
-        tx_out_index += 1;
     }
 
     let commitment_txout = TxOut {
diff --git a/src/database.rs b/src/database.rs
index 3c5dfda..77eb966 100644
--- a/src/database.rs
+++ b/src/database.rs
@@ -98,7 +98,10 @@ impl Database {
 
     pub fn save_proof(&self, proof: &Proof, txid: &Sha256dHash) {
         for out in &proof.output {
-            let outpoint_str = txid.be_hex_string() + ":" + out.get_vout().to_string().as_str();
+            let outpoint_str = match out.get_vout() {
+                Some(vout) => txid.be_hex_string() + ":" + vout.to_string().as_str(),
+                None => txid.be_hex_string() + ":BURN"
+            };
 
             let mut proof_path = self.basedir.clone();
             proof_path.push(outpoint_str);
diff --git a/src/kaleidoscope/burn.rs b/src/kaleidoscope/burn.rs
index 54c576c..3d53041 100644
--- a/src/kaleidoscope/burn.rs
+++ b/src/kaleidoscope/burn.rs
@@ -1,8 +1,12 @@
+use bifrost::upload_proofs;
 use bitcoin::network::constants::Network;
+use bitcoin::network::serialize::BitcoinHash;
 use bitcoin::OutPoint;
 use bitcoin::util::hash::Sha256dHash;
 use chain::indexer::fetch_transactions;
 use chain::tx_builder::{build_issuance_tx, raw_tx_commit_to};
+use chain::tx_builder::BitcoinRgbOutPoints;
+use chain::tx_builder::spend_proofs;
 use chain::wallet::*;
 use clap::ArgMatches;
 use database::Database;
@@ -11,42 +15,147 @@ use jsonrpc::client::Client;
 use kaleidoscope::{Config, RGBSubCommand};
 use kaleidoscope::sendtoaddress::send_to_address;
 use rgb::contract::Contract;
-use rgb::proof::OutputEntry;
+use rgb::output_entry::OutputEntry;
 use rgb::proof::Proof;
 use rgb::traits::Verify;
+use std::cmp;
 use std::collections::HashMap;
 
 pub struct Burn {}
 
-impl<'a> RGBSubCommand<'a> for Burn {
-    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
-        let asset_id = Sha256dHash::from_hex(matches.value_of("asset_id").unwrap()).unwrap();
-        let amount: u32 = matches.value_of("amount").unwrap().parse().unwrap();
+pub fn burn_tokens(server: &str, asset_id: Sha256dHash, amount: u32, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
+    const FEE: u64 = 2000;
+    let change_address = rpc_getnewaddress(client).unwrap();
 
-        let unspent_utxos = rpc_list_unspent(client).unwrap();
-        let mut burn_address = None;
-
-        // TODO: save contracts in the database to avoid looking for them like that
-        'outer: for (outpoint, amount) in unspent_utxos {
-            let proofs = database.get_proofs_for(&outpoint);
-
-            for p in proofs {
-                for entry in &p.output {
-                    if entry.get_vout() == outpoint.vout {
-                        if entry.get_asset_id() == asset_id {
-                            burn_address = Some(p.get_contract_for(asset_id.clone()).unwrap().burn_address);
-                            break 'outer;
-                        }
+    // -----------------
+
+    let unspent_utxos = rpc_list_unspent(client).unwrap();
+
+    let mut chosen_outpoints = Vec::new();
+    let mut chosen_proofs = Vec::new();
+    let mut total_btc_amount: u64 = 0;
+    let mut total_asset_amount: u32 = 0;
+    let mut to_self: HashMap<Sha256dHash, u32> = HashMap::new();
+
+    let mut used_proofs = HashMap::new();
+
+    for (outpoint, btc_amount) in &unspent_utxos {
+        let proofs = database.get_proofs_for(&outpoint);
+        let mut used = false;
+
+        // While theoretically there could be more proofs for the same outpoint,
+        // in this basic version the only way to bind some tokens to a UTXO
+        // is by actually creating it. Thus, since the same output cannot be created
+        // twice, we will always have at most one proof.
+
+        if proofs.len() == 0 {
+            continue;
+        }
+
+        let p = &proofs[0];
+
+        for entry in &p.output {
+            if entry.get_vout().is_some() && entry.get_vout().unwrap() == outpoint.vout { // entry for us
+                used = true;
+
+                if entry.get_asset_id() != asset_id { // full back to self, different asset
+                    let aggregator = to_self.entry(entry.get_asset_id()).or_insert(0);
+                    *aggregator += entry.get_amount();
+                } else {
+                    let use_from_this = cmp::min(
+                        amount - total_asset_amount, // remaining part
+                        entry.get_amount(), // or all of it
+                    );
+
+                    total_asset_amount += use_from_this;
+
+                    if use_from_this < entry.get_amount() { // partial back to self
+                        let aggregator = to_self.entry(entry.get_asset_id()).or_insert(0);
+                        *aggregator += entry.get_amount() - use_from_this;
                     }
                 }
             }
         }
 
-        if !burn_address.is_some() {
-            println!("Contract not found for {}", asset_id);
-            return Err(jsonrpc::Error::NoErrorOrResult);
+        if used {
+            total_btc_amount += btc_amount; // add the btc amount
+            chosen_outpoints.push(outpoint.clone()); // set as input for the tx
+
+            // Even though each output will only have (at most) one proof, it's still possible that
+            // multiple outputs share the same proof. This is why we need to keep track of the ones
+            // already spent.
+
+            if !used_proofs.get(&p.bitcoin_hash()).is_some() { // hasn't been used
+                chosen_proofs.push(p.clone()); // spend the proof
+                used_proofs.insert(p.bitcoin_hash(), true); // mark as used
+            }
+        }
+
+        if total_asset_amount == amount { // we are done here
+            break;
         }
+    }
+
+    if total_asset_amount < amount {
+        println!("Insufficient funds! {} < {}", total_asset_amount, amount);
+        return Err(jsonrpc::Error::NoErrorOrResult);
+    }
+
+    // --------------------------------------
+
+    let mut rgb_outputs = Vec::new();
+
+    total_btc_amount -= FEE;
+    let payment_amount = 0; // Do not burn any bitcoins
+
+    // 0 = payment
+    let mut payment_map = HashMap::new();
+    payment_map.insert(asset_id.clone(), amount);
+    rgb_outputs.push(BitcoinRgbOutPoints::new(None, payment_amount, payment_map));
+
+    // 1 = change
+    rgb_outputs.push(BitcoinRgbOutPoints::new(Some(change_address.clone()), total_btc_amount - payment_amount, to_self.clone()));
+
+    let (final_p, final_tx) = spend_proofs(&chosen_proofs, &chosen_outpoints, &rgb_outputs);
+
+    // ---------------------------------------
+
+    let final_tx = rpc_sign_transaction(client, &final_tx).unwrap();
+
+    println!("Created a new TX with the following outputs:");
+    // 0 = payment
+    println!("\t         {} of {} to {}", amount, asset_id, "BURN");
+    println!("\t         {} SAT to {}", payment_amount, "BURN");
+    // 1 = change
+    for (to_self_asset, to_self_amount) in &to_self {
+        println!("\t[CHANGE] {} of {} to {}", to_self_amount, to_self_asset, change_address.clone());
+    }
+    println!("\t[CHANGE] {} SAT to {}", total_btc_amount - payment_amount, change_address.clone());
+
+    println!("TXID: {}", final_tx.txid());
+
+    // ----------------------------------------
+
+    //println!("{:#?}", final_p);
+
+    // upload to server
+    upload_proofs(&String::from(server), &final_p, &final_tx.txid()).unwrap();
+    println!("Proof uploaded to {}", server);
+
+    database.save_proof(&final_p, &final_tx.txid());
+    rpc_broadcast(client, &final_tx);
+
+    Ok(())
+}
+
+impl<'a> RGBSubCommand<'a> for Burn {
+    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
+        let asset_id = Sha256dHash::from_hex(matches.value_of("asset_id").unwrap()).unwrap();
+        let amount: u32 = matches.value_of("amount").unwrap().parse().unwrap();
+
+        let unspent_utxos = rpc_list_unspent(client).unwrap();
+        let our_address = rpc_getnewaddress(client).unwrap();
 
-        send_to_address(burn_address.unwrap(), config.default_server.as_str(), asset_id, amount, config, database, client)
+        burn_tokens(config.default_server.as_str(), asset_id, amount, config, database, client)
     }
 }
\ No newline at end of file
diff --git a/src/kaleidoscope/getnewaddress.rs b/src/kaleidoscope/getnewaddress.rs
index fcc2dc8..dc36817 100644
--- a/src/kaleidoscope/getnewaddress.rs
+++ b/src/kaleidoscope/getnewaddress.rs
@@ -10,7 +10,7 @@ use jsonrpc;
 use jsonrpc::client::Client;
 use kaleidoscope::{Config, RGBSubCommand};
 use rgb::contract::Contract;
-use rgb::proof::OutputEntry;
+use rgb::output_entry::OutputEntry;
 use rgb::proof::Proof;
 use rgb::traits::Verify;
 use std::collections::HashMap;
diff --git a/src/kaleidoscope/issueasset.rs b/src/kaleidoscope/issueasset.rs
index 89b4988..73066a1 100644
--- a/src/kaleidoscope/issueasset.rs
+++ b/src/kaleidoscope/issueasset.rs
@@ -9,7 +9,7 @@ use jsonrpc;
 use jsonrpc::client::Client;
 use kaleidoscope::{Config, RGBSubCommand};
 use rgb::contract::Contract;
-use rgb::proof::OutputEntry;
+use rgb::output_entry::OutputEntry;
 use rgb::proof::Proof;
 use std::collections::HashMap;
 
@@ -65,12 +65,9 @@ impl<'a> RGBSubCommand<'a> for IssueAsset {
 
         // -------------------------------------
 
-        let burn_address = rpc_getnewaddress(client).unwrap();
-
         let contract = Contract {
             title: matches.value_of("title").unwrap().to_string(),
             total_supply: matches.value_of("total_supply").unwrap().parse().unwrap(),
-            burn_address,
             network,
             issuance_utxo,
             initial_owner_utxo,
@@ -94,7 +91,7 @@ impl<'a> RGBSubCommand<'a> for IssueAsset {
         let root_proof = Proof::new(
             vec![contract.initial_owner_utxo.clone()],
             vec![],
-            vec![OutputEntry::new(contract.get_asset_id(), contract.total_supply, 0)],
+            vec![OutputEntry::new(contract.get_asset_id(), contract.total_supply, Some(0))],
             Some(&contract));
 
         let root_proof_change_address = rpc_getnewaddress(client).unwrap();
diff --git a/src/kaleidoscope/listunspent.rs b/src/kaleidoscope/listunspent.rs
index 442c2c3..cb6ec23 100644
--- a/src/kaleidoscope/listunspent.rs
+++ b/src/kaleidoscope/listunspent.rs
@@ -10,7 +10,7 @@ use jsonrpc;
 use jsonrpc::client::Client;
 use kaleidoscope::{Config, RGBSubCommand};
 use rgb::contract::Contract;
-use rgb::proof::OutputEntry;
+use rgb::output_entry::OutputEntry;
 use rgb::proof::Proof;
 use rgb::traits::Verify;
 use std::collections::HashMap;
@@ -50,7 +50,7 @@ impl<'a> RGBSubCommand<'a> for ListUnspent {
                     // -------------------------
 
                     for entry in p.output {
-                        if entry.get_vout() == outpoint.vout {
+                        if entry.get_vout().is_some() && entry.get_vout().unwrap() == outpoint.vout {
                             println!("|  {}   |", entry.get_asset_id());
                             println!("|    Amount: {:12}                                             |", entry.get_amount());
                         }
diff --git a/src/kaleidoscope/sendtoaddress.rs b/src/kaleidoscope/sendtoaddress.rs
index 8b05502..7870c4b 100644
--- a/src/kaleidoscope/sendtoaddress.rs
+++ b/src/kaleidoscope/sendtoaddress.rs
@@ -13,7 +13,7 @@ use jsonrpc;
 use jsonrpc::client::Client;
 use kaleidoscope::{Config, RGBSubCommand};
 use rgb::contract::Contract;
-use rgb::proof::OutputEntry;
+use rgb::output_entry::OutputEntry;
 use rgb::proof::Proof;
 use std::cmp;
 use std::collections::HashMap;
@@ -53,7 +53,7 @@ pub fn send_to_address(btc_address: Address, server: &str, asset_id: Sha256dHash
         let p = &proofs[0];
 
         for entry in &p.output {
-            if entry.get_vout() == outpoint.vout { // entry for us
+            if entry.get_vout().is_some() && entry.get_vout().unwrap() == outpoint.vout { // entry for us
                 used = true;
 
                 if entry.get_asset_id() != asset_id { // full back to self, different asset
@@ -109,10 +109,10 @@ pub fn send_to_address(btc_address: Address, server: &str, asset_id: Sha256dHash
     // 0 = payment
     let mut payment_map = HashMap::new();
     payment_map.insert(asset_id.clone(), amount);
-    rgb_outputs.push(BitcoinRgbOutPoints::new(btc_address.clone(), payment_amount, payment_map));
+    rgb_outputs.push(BitcoinRgbOutPoints::new(Some(btc_address.clone()), payment_amount, payment_map));
 
     // 1 = change
-    rgb_outputs.push(BitcoinRgbOutPoints::new(change_address.clone(), total_btc_amount - payment_amount, to_self.clone()));
+    rgb_outputs.push(BitcoinRgbOutPoints::new(Some(change_address.clone()), total_btc_amount - payment_amount, to_self.clone()));
 
     let (final_p, final_tx) = spend_proofs(&chosen_proofs, &chosen_outpoints, &rgb_outputs);
 
diff --git a/src/kaleidoscope/sync.rs b/src/kaleidoscope/sync.rs
index 39a54d8..b764c90 100644
--- a/src/kaleidoscope/sync.rs
+++ b/src/kaleidoscope/sync.rs
@@ -13,7 +13,7 @@ use jsonrpc;
 use jsonrpc::client::Client;
 use kaleidoscope::{Config, RGBSubCommand};
 use rgb::contract::Contract;
-use rgb::proof::OutputEntry;
+use rgb::output_entry::OutputEntry;
 use rgb::proof::Proof;
 use rgb::traits::Verify;
 use std::collections::HashMap;
