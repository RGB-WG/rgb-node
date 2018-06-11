use bitcoin::blockdata::transaction::TxOutRef;
use entities::proofs::Proof;
use entities::rgb_output::RgbOutput;
use entities::traits::Verify;
use std::collections::{HashMap, HashSet};
use bitcoin::blockdata::transaction::Transaction;
use util::error::Error::{MissingTx, LoopDetected};
use util;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::util::hash::Sha256dHash;
use entities::traits::ContainsSignatures;

#[derive(Clone, Debug)]
pub struct ProofHeader {
    pub bind_to: TxOutRef,
    pub inputs: Vec<Proof>,
    pub outputs: Vec<RgbOutput>,
    pub signatures: Vec<Vec<u8>>,
    // TODO: look for a struct in rust-secp256k1
    pub tx_commitment_r: Vec<Vec<u8>>, // TODO: once again, look for a struct in rust-secp256k1
}

impl ProofHeader {
    pub fn new(bind_to: TxOutRef, inputs: &[Proof], outputs: &[RgbOutput]) -> ProofHeader {
        ProofHeader {
            bind_to,
            inputs: inputs.to_vec(),
            outputs: outputs.to_vec(),
            signatures: Vec::new(),
            tx_commitment_r: Vec::new(),
        }
    }
}

impl Verify for ProofHeader {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>) -> Result<(), util::error::Error> {
        // TODO: verify the commitment to this proof

        let tx_committing_to_this = utxos_spent_in.get(&self.bind_to);
        if tx_committing_to_this == None {
            return Err(MissingTx(self.bind_to));
        }

        // Make sure that there are no loops
        for input in &self.inputs {
            // TODO: prove that loops can only be "1 proof long". Because of the commitments in the
            // ..... UTXOs, I feel like there's no way to make a loop longer than that, but I might
            // ..... be wrong.

            // same `bind_to` for two consecutive proofs
            if input.header.bind_to == self.bind_to {
                return Err(LoopDetected(self.bind_to));
            }
        }

        Ok(())
    }

    fn get_necessary_txs(&self, set: &mut HashSet<TxOutRef>) {
        set.insert(self.bind_to);

        for input_proof in self.inputs.iter() {
            input_proof.get_necessary_txs(set);
        }
    }
}

impl ContainsSignatures for ProofHeader {
    fn push_signature(&mut self, signature: Vec<u8>, s2c_commitment: Vec<u8>) {
        self.signatures.push(signature);
        self.tx_commitment_r.push(s2c_commitment);
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for ProofHeader {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Encode the version (16 bit so that we can use some bits as switches if necessary in the future)
        (1u16).consensus_encode(s)?;

        // Encode the `bind_to` (TxOutRef)
        self.bind_to.txid.consensus_encode(s)?;
        (self.bind_to.index as u32).consensus_encode(s)?; // TODO: Is the casting here really necessary?

        // Encode the outputs
        self.outputs.consensus_encode(s)?;

        // Encode the signatures and the R used to verify the commitments.
        self.signatures.consensus_encode(s)?;
        self.tx_commitment_r.consensus_encode(s)?;

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for ProofHeader {
    fn consensus_decode(d: &mut D) -> Result<ProofHeader, D::Error> {
        let _version: u16 = ConsensusDecodable::consensus_decode(d)?;

        let bind_to_txid: Sha256dHash = ConsensusDecodable::consensus_decode(d)?;
        let bind_to_index: u32 = ConsensusDecodable::consensus_decode(d)?;

        let bind_to = TxOutRef {
            txid: bind_to_txid,
            index: bind_to_index as usize,
        };

        let outputs: Vec<RgbOutput> = ConsensusDecodable::consensus_decode(d)?;

        let signatures: Vec<Vec<u8>> = ConsensusDecodable::consensus_decode(d)?;
        let tx_commitment_r: Vec<Vec<u8>> = ConsensusDecodable::consensus_decode(d)?;

        Ok(ProofHeader {
            bind_to,
            inputs: Vec::new(), // TODO!!
            outputs,
            signatures,
            tx_commitment_r,
        })
    }
}