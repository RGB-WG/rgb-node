use bitcoin::{Transaction, TxIn, TxOut};
use bitcoin::Address;
use bitcoin::blockdata::script::Script;
use bitcoin::OutPoint;
use bitcoin::util::hash::Sha256dHash;
use rgb::contract::Contract;
use rgb::output_entry::OutputEntry;
use rgb::proof::Proof;
use rgb::traits::Verify;
use std::collections::HashMap;

pub fn build_issuance_tx(contract: &Contract, outputs: &HashMap<Address, u64>) -> Transaction {
    let txin = TxIn {
        previous_output: contract.issuance_utxo,
        script_sig: Script::default(),
        sequence: 0,
        witness: Vec::new(),
    };

    let mut tx_outs = Vec::new();

    for output in outputs {
        let this_tx_out = TxOut {
            value: *output.1,
            script_pubkey: output.0.script_pubkey(),
        };

        tx_outs.push(this_tx_out);
    }

    let commitment_txout = TxOut {
        value: 0,
        script_pubkey: contract.get_expected_script(),
    };

    tx_outs.push(commitment_txout);

    Transaction {
        version: 1,
        lock_time: 0,
        input: vec![txin],
        output: tx_outs,
    }
}

#[derive(Clone, Debug)]
pub struct BitcoinRgbOutPoints {
    pub bitcoin_address: Option<Address>,
    pub bitcoin_amount: u64,
    pub rgb_outputs: HashMap<Sha256dHash, u32>,
}

impl BitcoinRgbOutPoints {
    pub fn new(bitcoin_address: Option<Address>, bitcoin_amount: u64, rgb_outputs: HashMap<Sha256dHash, u32>) -> BitcoinRgbOutPoints {
        BitcoinRgbOutPoints {
            bitcoin_address,
            bitcoin_amount,
            rgb_outputs,
        }
    }
}

pub fn spend_proofs(input_proofs: &Vec<Proof>, bitcoin_inputs: &Vec<OutPoint>, outputs: &Vec<BitcoinRgbOutPoints>) -> (Proof, Transaction) {
    // TODO: use raw_tx_commit_to
    // Create all the inputs of this transaction by iterating the outputs of the previous one(s)

    let mut tx_ins = Vec::new();
    let mut bind_to = Vec::new();

    for out_point in bitcoin_inputs {
        let this_txin = TxIn {
            previous_output: out_point.clone(),
            script_sig: Script::default(),
            sequence: 0,
            witness: Vec::new(),
        };

        tx_ins.push(this_txin);
        bind_to.push(out_point.clone());
    }

    // ------------------------------------------
    // Prepare the partial prooof (no outputs)

    let mut proof = Proof {
        bind_to,
        input: input_proofs.clone(),
        output: Vec::new(),
        contract: None,
    };

    // ------------------------------------------
    // Create all the outputs of this transaction

    let mut tx_outs = Vec::new();
    let mut tx_out_index = 0;

    for output_item in outputs {
        match output_item.bitcoin_address {
            Some(ref addr) => {
                let this_tx_out = TxOut {
                    value: output_item.bitcoin_amount,
                    script_pubkey: addr.script_pubkey(),
                };

                tx_outs.push(this_tx_out);

                for (asset_id, amount) in &output_item.rgb_outputs {
                    proof.output.push(OutputEntry::new(asset_id.clone(), amount.clone(), Some(tx_out_index)));
                }

                tx_out_index += 1;
            },
            None => {
                for (asset_id, amount) in &output_item.rgb_outputs {
                    proof.output.push(OutputEntry::new(asset_id.clone(), amount.clone(), None));
                }
            }
        }
    }

    let commitment_txout = TxOut {
        value: 0,
        script_pubkey: proof.get_expected_script(),
    };

    tx_outs.push(commitment_txout);

    (proof, Transaction {
        version: 1,
        lock_time: 0,
        input: tx_ins,
        output: tx_outs,
    })
}

pub fn raw_tx_commit_to(proof: &Proof, inputs: Vec<OutPoint>, outputs: &HashMap<Address, u64>) -> Transaction {
    // Create all the inputs of this transaction by iterating the outputs of the previous one(s)

    let mut tx_ins = Vec::new();

    for out_point in inputs {
        let this_txin = TxIn {
            previous_output: out_point.clone(),
            script_sig: Script::default(),
            sequence: 0,
            witness: Vec::new(),
        };

        tx_ins.push(this_txin);
    }

    let mut tx_outs = Vec::new();

    for (addr, amount) in outputs {
        let this_tx_out = TxOut {
            value: *amount,
            script_pubkey: addr.script_pubkey(),
        };

        tx_outs.push(this_tx_out);
    }

    let commitment_txout = TxOut {
        value: 0,
        script_pubkey: proof.get_expected_script(),
    };

    tx_outs.push(commitment_txout);

    Transaction {
        version: 1,
        lock_time: 0,
        input: tx_ins,
        output: tx_outs,
    }
}