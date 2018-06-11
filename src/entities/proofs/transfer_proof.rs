use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::util::uint::Uint256;
use entities::proofs::Proof;
use entities::proofs::header::ProofHeader;
use entities::rgb_output::RgbOutPoint;
use entities::rgb_output::RgbOutPoint::KnownUTXO;
use entities::rgb_output::RgbOutPoint::NewUTXO;
use entities::rgb_output::RgbOutput;
use entities::traits::Verify;
use entities::traits::VerifyData;
use std::collections::HashMap;
use util;
use util::error::Error::MissingTx;
use util::error::Error::InvalidOutputIndex;
use util::error::Error::InputOutputMismatch;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::encodable::ConsensusDecodable;
use entities::proofs::data::ProofData::BranchTranferProof;
use bitcoin::util::hash::Sha256dHash;
use util::txs::BuildFromTx;
use util::error::Error::WrongOutputSpentEntry;
use util::txs::VerifyInputs;

#[derive(Clone, Debug)]
pub struct TransferProofData {}

impl TransferProofData {
    pub fn new_data() -> TransferProofData {
        TransferProofData {}
    }

    pub fn new(bind_to: TxOutRef, inputs: &[Proof], outputs: &[RgbOutput]) -> Proof {
        Proof::new(
            ProofHeader::new(bind_to, inputs, outputs),
            BranchTranferProof(TransferProofData::new_data()),
        )
    }
}

impl VerifyData for TransferProofData {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>, header: &ProofHeader) -> Result<(), util::error::Error> {
        // TODO: check for min_amount, max_hops from each contract.

        let mut inputs: HashMap<&Sha256dHash, u64> = HashMap::new();
        for input_proof in header.inputs.iter() {
            // Verify the previous input proof
            if let Err(e) = input_proof.verify(utxos_spent_in) {
                return Err(e);
            }

            let tx_committing_to_input = utxos_spent_in.get(&input_proof.header.bind_to);
            if tx_committing_to_input == None {
                return Err(MissingTx(header.bind_to));
            }
            let tx_committing_to_input = tx_committing_to_input.unwrap();

            if !tx_committing_to_input.is_input(&input_proof.header.bind_to) {
                return Err(WrongOutputSpentEntry(input_proof.header.bind_to));
            }

            for input in input_proof.header.outputs.iter() {
                match input.to_output {
                    KnownUTXO(utxo_hash) if utxo_hash != RgbOutPoint::hash_txoutref(header.bind_to) => {
                        // Not for us, skip this output
                        continue;
                    }
                    NewUTXO(index) if TxOutRef::from_tx(tx_committing_to_input, index as usize).unwrap() != header.bind_to => {
                        // Not for us, skip this output
                        continue;
                    },
                    _ => {} // go on
                }

                *inputs.entry(&input.token_id).or_insert(0) += input.amount as u64;
            }
        }

        let mut outputs: HashMap<&Sha256dHash, u64> = HashMap::new();
        for output in header.outputs.iter() {
            *outputs.entry(&output.token_id).or_insert(0) += output.amount as u64;
        }

        if inputs == outputs {
            Ok(())
        } else {
            Err(InputOutputMismatch)
        }
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for TransferProofData {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Add the kind of proof
        1u8.consensus_encode(s)?;

        // That's it actually
        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for TransferProofData {
    fn consensus_decode(d: &mut D) -> Result<TransferProofData, D::Error> {
        Ok(TransferProofData {})
    }
}