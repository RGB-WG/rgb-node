use bitcoin::BitcoinHash;
use bitcoin::blockdata::opcodes::*;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::script::Script;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::Transaction;
use bitcoin::util::hash::Sha256dHash;
use contract::Contract;
use output_entry::OutputEntry;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use super::bitcoin::OutPoint;
use super::traits::Verify;
use traits::NeededTx;

#[derive(Clone, Debug)]
pub struct Proof {
    /// The spent assets are held by these txos
    pub bind_to: Vec<OutPoint>,
    /// All the proofs of inputs txs
    pub input: Vec<Proof>,
    /// RGB outputs. If output entry.vout is None then use the index in this vector
    pub output: Vec<OutputEntry>,
    /// Issuance contract, only needed for root proofs
    pub contract: Option<Box<Contract>>,
}

impl Proof {
    pub fn new(bind_to: Vec<OutPoint>, input: Vec<Proof>, output: Vec<OutputEntry>, contract: Option<&Contract>) -> Proof {
        let contract = if contract.is_some() { Some(Box::new(contract.unwrap().clone())) } else { None };

        Proof {
            bind_to,
            input,
            output,
            contract,
        }
    }

    pub fn is_root_proof(&self) -> bool {
        return self.contract.is_some() && self.bind_to.len() == 1 && self.bind_to[0] == self.contract.as_ref().unwrap().initial_owner_utxo;
    }

    /// Look for test_proof OutputEntries spent in the first elements of self.bind_to,
    /// if test_proof is a first level parent of the tx associated to this proof (?)
    fn get_entries_for_us(&self, test_proof: &Proof, needed_txs: &HashMap<&NeededTx, Transaction>) -> Vec<OutputEntry> {
        // We know that [0] is equal to all others (checked in verify)
        let committing_tx_this = needed_txs.get(&NeededTx::WhichSpendsOutPoint(self.bind_to[0])).unwrap();
        let committing_tx_test = needed_txs.get(&NeededTx::WhichSpendsOutPoint(test_proof.bind_to[0])).unwrap();

        let mut ans = Vec::new();

        for i in 0..committing_tx_this.input.len() {
            if committing_tx_this.input[i].previous_output.txid != committing_tx_test.txid() {
                // Not from the input proof we are processing now, ignoring it
                continue;
            }

            // The output index contained in previous_output is for us
            let input_for_us = committing_tx_this.input[i].previous_output.vout;

            for entry in &test_proof.output {
                if entry.get_vout().is_some() && entry.get_vout().unwrap() == input_for_us {
                    ans.push(entry.clone());
                }
            }
        }

        ans
    }
}

impl BitcoinHash for Proof {
    fn bitcoin_hash(&self) -> Sha256dHash {
        // only need to hash the outputs
        use bitcoin::network::serialize::serialize;
        let encoded = serialize(&self.output).unwrap();

        Sha256dHash::from_data(&encoded)
    }
}

impl Verify for Proof {
    fn get_needed_txs(&self) -> Vec<NeededTx> {
        let mut ans = Vec::new();

        for out_point in &self.bind_to {
            ans.push(NeededTx::WhichSpendsOutPoint(out_point.clone()));
        }

        if self.is_root_proof() {
            let mut needed_txs = self.contract.as_ref().unwrap().get_needed_txs();
            ans.append(&mut needed_txs);
        } else {
            for i in 0..self.input.len() { // iterate the input proofs
                let mut needed_txs = self.input[i].get_needed_txs();
                ans.append(&mut needed_txs);
            }
        }

        ans
    }

    fn verify(&self, needed_txs: &HashMap<&NeededTx, Transaction>) -> bool {
        // Make sure that all the outpoints we are binding to are spent in the same tx

        let committing_tx = needed_txs.get(&NeededTx::WhichSpendsOutPoint(self.bind_to[0])).unwrap(); // Take the first one
        for out_point in &self.bind_to {
            // And compare it to all the others
            let this_committing_tx = needed_txs.get(&NeededTx::WhichSpendsOutPoint(out_point.clone())).unwrap();

            if committing_tx.txid() != this_committing_tx.txid() {
                println!("not all the outpoints in bind_to are spent in the same tx {:?}", committing_tx.txid());
                return false;
            }
        }

        // ---------------------------------

        let expected = self.get_expected_script();

        // Check the tx outputs for the commitment
        let mut found_output = false;
        for i in 0..committing_tx.output.len() {
            found_output = found_output || committing_tx.output[i].script_pubkey == expected;
        }

        if !found_output {
            println!("invalid commitment");
            return false;
        }

        // --------------------------------------------------------

        let mut in_amounts = HashMap::new();

        if self.is_root_proof() {
            if self.input.len() > 0 {
                println!("the root proof should not have any input proofs");
                return false;
            }

            in_amounts.insert(self.contract.as_ref().unwrap().get_asset_id(), self.contract.as_ref().unwrap().total_supply);
        } else {
            let mut in_entries = Vec::new();

            for input_proof in &self.input {
                let mut entries_for_us = self.get_entries_for_us(input_proof, &needed_txs);
                in_entries.append(&mut entries_for_us);
            }

            // Aggregate the amounts
            for entry in in_entries {
                let aggregator = in_amounts.entry(entry.get_asset_id()).or_insert(0);
                *aggregator += entry.get_amount();
            }
        }

        // --------------------------------------------------------

        // Check the amounts
        let mut out_amounts = HashMap::new();

        for output_entry in &self.output {
            let aggregator = out_amounts.entry(output_entry.get_asset_id()).or_insert(0);
            *aggregator += output_entry.get_amount();
        }

        if in_amounts != out_amounts {
            println!("input/output mismatch: {:?} {:?}", in_amounts, out_amounts);
            return false;
        }

        true
    }

    fn get_expected_script(&self) -> Script {
        let burn_script_builder = Builder::new();

        let burn_script_builder = burn_script_builder.push_opcode(All::OP_RETURN);
        let burn_script_builder = burn_script_builder.push_slice(self.bitcoin_hash().as_bytes());

        burn_script_builder.into_script()
    }
}

impl PartialEq for Proof {
    fn eq(&self, other: &Proof) -> bool {
        self.bitcoin_hash() == other.bitcoin_hash()
    }
}

impl Eq for Proof {}

impl Hash for Proof {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let consensus_hash = self.bitcoin_hash();
        consensus_hash.hash(state);
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for Proof {
    fn consensus_encode(&self, s: &mut S) -> Result<(), serialize::Error> {
        self.bind_to.consensus_encode(s)?;
        self.input.consensus_encode(s)?;
        self.output.consensus_encode(s)?;
        self.contract.consensus_encode(s)
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for Proof {
    fn consensus_decode(d: &mut D) -> Result<Proof, serialize::Error> {
        Ok(Proof {
            bind_to: ConsensusDecodable::consensus_decode(d)?,
            input: ConsensusDecodable::consensus_decode(d)?,
            output: ConsensusDecodable::consensus_decode(d)?,
            contract: ConsensusDecodable::consensus_decode(d)?,
        })
    }
}