use bitcoin::BitcoinHash;
use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::script::Script;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::Transaction;
use bitcoin::util::hash::Hash160;
use bitcoin::util::hash::Sha256dHash;
use contract::Contract;
use pay_to_contract::ECTweakFactor;
use secp256k1::Error;
use secp256k1::PublicKey;
use secp256k1::Secp256k1;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use super::bitcoin::OutPoint;
use super::traits::Verify;
use traits::NeededTx;
use traits::PayToContract;

#[derive(Clone, Debug)]
pub struct OutputEntry(Sha256dHash, u32, u32); // asset_id, amount -> vout

impl OutputEntry {
    pub fn new(asset_id: Sha256dHash, amount: u32, vout: u32) -> OutputEntry {
        OutputEntry(asset_id, amount, vout)
    }

    pub fn get_asset_id(&self) -> Sha256dHash {
        self.0.clone()
    }

    pub fn get_amount(&self) -> u32 {
        self.1
    }

    pub fn get_vout(&self) -> u32 {
        self.2
    }
}

#[derive(Clone, Debug)]
pub struct Proof {
    pub bind_to: Vec<OutPoint>,
    pub input: Vec<Proof>,
    pub output: Vec<OutputEntry>,
    pub contract: Option<Box<Contract>>, // Only needed for root proofs
    pub original_commitment_pk: Option<PublicKey>
}

impl Proof {
    pub fn new(bind_to: Vec<OutPoint>, input: Vec<Proof>, output: Vec<OutputEntry>, contract: Option<&Contract>, original_commitment_pk: Option<PublicKey>) -> Proof {
        let contract = if contract.is_some() { Some(Box::new(contract.unwrap().clone())) } else { None };

        Proof {
            bind_to,
            input,
            output,
            contract,
            original_commitment_pk
        }
    }

    pub fn is_root_proof(&self) -> bool {
        return self.contract.is_some() && self.bind_to.len() == 1 && self.bind_to[0] == self.contract.as_ref().unwrap().initial_owner_utxo;
    }

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
                if entry.2 == input_for_us {
                    ans.push(entry.clone());
                }
            }
        }

        ans
    }

    pub fn get_contract_for(&self, asset_id: Sha256dHash) -> Option<Contract> {
        if self.is_root_proof() && self.contract.as_ref().unwrap().get_asset_id() == asset_id {
            return Some(self.contract.as_ref().unwrap().as_ref().clone());
        } else {
            for input in &self.input {
                let result = input.get_contract_for(asset_id);
                if result.is_some() {
                    return result;
                }
            }
        }

        None
    }
}

impl BitcoinHash for Proof {
    fn bitcoin_hash(&self) -> Sha256dHash {
        // only need to hash the outputs
        // TODO: do we need to commit to the original pubkey? (pretty sure it's a "no")

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

        // TODO: right now we are forcing the commitment to be in the first output
        if committing_tx.output[0].script_pubkey != self.get_expected_script() {
            println!("invalid commitment");
            return false;
        }

        // --------------------------------------------------------

        let mut in_amounts = HashMap::new();

        if self.is_root_proof() {
            // Burn addresses are only checked in normal proofs, not root proofs

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

                // -------------------------------------------------------
                // Make sure we are not spending burned assets

                let tx_spent = needed_txs.get(&NeededTx::WhichSpendsOutPoint(input_proof.bind_to[0])).unwrap();
                for entry in &entries_for_us {
                    let index: usize = entry.get_vout() as usize;
                    let script_pubkey = &tx_spent.output[index].script_pubkey;

                    if script_pubkey == &self.get_contract_for(entry.get_asset_id()).unwrap().burn_address.script_pubkey() {
                        println!("Trying to spend burned coins!");
                        return false;
                    }
                }
            }

            // Aggregate the amounts
            for entry in in_entries {
                let aggregator = in_amounts.entry(entry.0).or_insert(0);
                *aggregator += entry.1;
            }
        }

        // --------------------------------------------------------

        // Check the amounts
        let mut out_amounts = HashMap::new();

        for output_entry in &self.output {
            let aggregator = out_amounts.entry(output_entry.0).or_insert(0);
            *aggregator += output_entry.1;
        }

        if in_amounts != out_amounts {
            println!("input/output mismatch: {:?} {:?}", in_amounts, out_amounts);
            return false;
        }

        true
    }

    fn get_expected_script(&self) -> Script {
        let mut contract_pubkey = self.original_commitment_pk.unwrap().clone();

        let s = Secp256k1::new();
        self.get_self_tweak_factor().unwrap().add_to_pk(&s, &mut contract_pubkey).unwrap();

        Builder::new()
            .push_opcode(opcodes::All::OP_DUP)
            .push_opcode(opcodes::All::OP_HASH160)
            .push_slice(&(Hash160::from_data(&contract_pubkey.serialize()[..])[..]))
            .push_opcode(opcodes::All::OP_EQUALVERIFY)
            .push_opcode(opcodes::All::OP_CHECKSIG)
            .into_script()
    }
}

impl PayToContract for Proof {
    fn set_commitment_pk(&mut self, pk: &PublicKey) -> (PublicKey, ECTweakFactor) {
        self.original_commitment_pk = Some(pk.clone()); // set the original pk

        let s = Secp256k1::new();

        let mut new_pk = pk.clone();
        let tweak_factor = self.get_self_tweak_factor().unwrap();
        tweak_factor.add_to_pk(&s, &mut new_pk).unwrap();

        (new_pk, tweak_factor)
    }

    fn get_self_tweak_factor(&self) -> Result<ECTweakFactor, Error> {
        let s = Secp256k1::new();

        ECTweakFactor::from_pk_data(&s, &self.original_commitment_pk.unwrap(), &self.bitcoin_hash())
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

impl<S: SimpleEncoder> ConsensusEncodable<S> for OutputEntry {
    fn consensus_encode(&self, s: &mut S) -> Result<(), serialize::Error> {
        self.0.consensus_encode(s)?;
        self.1.consensus_encode(s)?;
        self.2.consensus_encode(s)
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for OutputEntry {
    fn consensus_decode(d: &mut D) -> Result<OutputEntry, serialize::Error> {
        Ok(OutputEntry::new(ConsensusDecodable::consensus_decode(d)?, ConsensusDecodable::consensus_decode(d)?, ConsensusDecodable::consensus_decode(d)?))
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for Proof {
    fn consensus_encode(&self, s: &mut S) -> Result<(), serialize::Error> {
        self.bind_to.consensus_encode(s)?;
        self.input.consensus_encode(s)?;
        self.output.consensus_encode(s)?;
        self.contract.consensus_encode(s)?;

        let original_commitment_pk_ser: Vec<u8> = match self.original_commitment_pk {
            Some(pk) => {
                let mut vec = Vec::with_capacity(33);
                vec.extend_from_slice(&pk.serialize());

                vec
            },
            None => Vec::new()
        };
        original_commitment_pk_ser.consensus_encode(s)
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for Proof {
    fn consensus_decode(d: &mut D) -> Result<Proof, serialize::Error> {
        let mut p = Proof {
            bind_to: ConsensusDecodable::consensus_decode(d)?,
            input: ConsensusDecodable::consensus_decode(d)?,
            output: ConsensusDecodable::consensus_decode(d)?,
            contract: ConsensusDecodable::consensus_decode(d)?,
            original_commitment_pk: None
        };

        let original_commitment_pk_ser: Vec<u8> = ConsensusDecodable::consensus_decode(d)?;
        if original_commitment_pk_ser.len() > 0 {
            let s = Secp256k1::new();
            p.set_commitment_pk(&PublicKey::from_slice(&s, &original_commitment_pk_ser.as_slice()).unwrap());
        }

        Ok(p)
    }
}