use bitcoin::{Transaction, TxIn, TxOut};
use bitcoin::Address;
use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::{Script, Builder};
use bitcoin::OutPoint;
use bitcoin::util::hash::Sha256dHash;
use rgb::contract::Contract;
use rgb::pay_to_contract::ECTweakFactor;
use rgb::output_entry::OutputEntry;
use rgb::proof::Proof;
use rgb::traits::PayToContract;
use rgb::traits::Verify;
use secp256k1::PublicKey;
use std::collections::HashMap;

pub fn build_issuance_tx(contract: &mut Contract, commitment_pubkey: &PublicKey, commitment_amount: u64, extra_outputs: &HashMap<Address, u64>) -> (Transaction, ECTweakFactor) {
    let txin = TxIn {
        previous_output: contract.issuance_utxo,
        script_sig: Script::default(),
        sequence: 0,
        witness: Vec::new(),
    };

    let mut tx_outs = Vec::new();

    let (_, tweak_factor) = contract.set_commitment_pk(commitment_pubkey);

    // Tx out first
    let commitment_txout = TxOut {
        value: commitment_amount,
        script_pubkey: contract.get_expected_script(),
    };

    tx_outs.push(commitment_txout);

    for output in extra_outputs {
        let this_tx_out = TxOut {
            value: *output.1,
            script_pubkey: output.0.script_pubkey(),
        };

        tx_outs.push(this_tx_out);
    }

    (Transaction {
        version: 1,
        lock_time: 0,
        input: vec![txin],
        output: tx_outs,
    }, tweak_factor)
}

#[derive(Clone, Debug)]
pub struct BitcoinRgbOutPoints {
    pub bitcoin_address: Option<Address>,
    pub bitcoin_amount: u64,
    pub rgb_outputs: HashMap<Sha256dHash, u64>,
}

impl BitcoinRgbOutPoints {
    // bitcoin_address can be None in case you want to burn an asset
    pub fn new(bitcoin_address: Option<Address>, bitcoin_amount: u64, rgb_outputs: HashMap<Sha256dHash, u64>) -> BitcoinRgbOutPoints {
        BitcoinRgbOutPoints {
            bitcoin_address,
            bitcoin_amount,
            rgb_outputs,
        }
    }
}

pub fn spend_proofs(input_proofs: &Vec<Proof>, bitcoin_inputs: &Vec<OutPoint>, outputs: &Vec<BitcoinRgbOutPoints>) -> (Proof, Transaction) {
    // ------------------------------------------
    // Prepare the partial prooof (no outputs)

    let mut proof = Proof {
        bind_to: bitcoin_inputs.clone(),
        input: input_proofs.clone(),
        output: Vec::new(),
        contract: None,
        original_commitment_pk: None
    };

    // ------------------------------------------
    // Create all the outputs of this proof and a map of the Bitcoin outputs

    let mut bitcoin_outputs = HashMap::new();
    let mut tx_out_index = 0;

    for output_item in outputs {
        match output_item.bitcoin_address {
            Some(ref addr) => {
                bitcoin_outputs.insert(addr.clone(), output_item.bitcoin_amount);

                for (asset_id, amount) in &output_item.rgb_outputs {
                    proof.output.push(OutputEntry::new(asset_id.clone(), amount.clone(), Some(tx_out_index)));
                }

                tx_out_index += 1;
            },
            None => {
                // Just burn this output

                for (asset_id, amount) in &output_item.rgb_outputs {
                    proof.output.push(OutputEntry::new(asset_id.clone(), amount.clone(), None));
                }
            }
        }
    }

    let tx = raw_tx_commit_to(&proof, bitcoin_inputs.clone(), &bitcoin_outputs);

    (proof, tx)
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

pub fn spend_proofs_p2c(input_proofs: &Vec<Proof>, bitcoin_inputs: &Vec<OutPoint>, commitment_pubkey: &PublicKey, commitment_amount: u64, commitment_tokens: &HashMap<Sha256dHash, u64>, other_outputs: &Vec<BitcoinRgbOutPoints>) -> (Proof, Transaction, ECTweakFactor) {
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
        original_commitment_pk: None
    };

    // ------------------------------------------
    // Create all the outputs of this transaction

    let mut tx_outs = Vec::new();

    // Commitment always goes first
    let commitment_txout = TxOut {
        value: commitment_amount,
        script_pubkey: Script::default(), // will be updated later
    };

    tx_outs.push(commitment_txout);

    for (asset_id, amount) in commitment_tokens {
        proof.output.push(OutputEntry::new(asset_id.clone(), amount.clone(), Some(0)));
    }

    // Add all the other outputs

    let mut tx_out_index = 1;

    for output_item in other_outputs {
        let script_pubkey = match output_item.bitcoin_address {
            Some(ref addr) => addr.script_pubkey(),
            None => Builder::new().push_opcode(opcodes::All::OP_RETURN).into_script()
        };

        let this_tx_out = TxOut {
            value: output_item.bitcoin_amount,
            script_pubkey,
        };

        tx_outs.push(this_tx_out);

        // Add the RGB outpoints
        for (asset_id, amount) in &output_item.rgb_outputs {
            proof.output.push(OutputEntry::new(asset_id.clone(), amount.clone(), Some(tx_out_index)));
        }

        tx_out_index += 1;
    }

    // Pay to contract
    let (_, tweak_factor) = proof.set_commitment_pk(commitment_pubkey);
    tx_outs[0].script_pubkey = proof.get_expected_script(); // updated commitment script after all the outpoints have been added

    (proof, Transaction {
        version: 1,
        lock_time: 0,
        input: tx_ins,
        output: tx_outs,
    }, tweak_factor)
}

pub fn raw_tx_commit_to_p2c(proof: &mut Proof, inputs: Vec<OutPoint>, commitment_pubkey: &PublicKey, commitment_amount: u64, other_outputs: &HashMap<Address, u64>) -> (Transaction, ECTweakFactor) {
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

    let (_, tweak_factor) = proof.set_commitment_pk(commitment_pubkey);

    let mut tx_outs = Vec::new();

    // Always the first one
    let commitment_txout = TxOut {
        value: commitment_amount,
        script_pubkey: proof.get_expected_script(),
    };

    tx_outs.push(commitment_txout);

    for (addr, amount) in other_outputs {
        let this_tx_out = TxOut {
            value: *amount,
            script_pubkey: addr.script_pubkey(),
        };

        tx_outs.push(this_tx_out);
    }

    (Transaction {
        version: 1,
        lock_time: 0,
        input: tx_ins,
        output: tx_outs,
    }, tweak_factor)
}