use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleEncoder;
use entities::contracts::Contract;
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
use util::error::Error::EmissionProofHasInputs;
use util::error::Error::InvalidEmissionOwner;
use util::error::Error::InputOutputMismatch;
use util::error::Error::WrongContractKind;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::encodable::ConsensusDecodable;
use entities::proofs::data::ProofData::BranchEmissionContractProof;
use entities::contracts::data::ContractData::EmissionData;
use bitcoin::util::hash::Sha256dHash;
use util::txs::BuildFromTx;
use util::error::Error::WrongOutputSpentEntry;
use util::txs::VerifyInputs;

#[derive(Clone, Debug)]
pub struct EmissionContractProofData {
    pub contract: Contract
}

impl EmissionContractProofData {
    pub fn new_data(contract: Contract) -> EmissionContractProofData {
        EmissionContractProofData {
            contract
        }
    }

    pub fn new(bind_to: TxOutRef, inputs: &[Proof], outputs: &[RgbOutput], contract: Contract) -> Proof {
        Proof::new(
            ProofHeader::new(bind_to, inputs, outputs),
            BranchEmissionContractProof(EmissionContractProofData::new_data(contract)),
        )
    }
}

impl VerifyData for EmissionContractProofData {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>, header: &ProofHeader) -> Result<(), util::error::Error> {
        if header.inputs.len() > 0 {
            return Err(EmissionProofHasInputs(header.inputs.len()));
        }

        // Check the kind of contract
        match self.contract.data {
            EmissionData(ref contract_data) => {
                // TODO: check for loops

                // Verify the contract
                if let Err(e) = self.contract.verify(utxos_spent_in) {
                    return Err(e);
                }

                // Check `owner_utxo`.
                match contract_data.owner_utxo {
                    KnownUTXO(utxo_hash) if utxo_hash != RgbOutPoint::hash_txoutref(header.bind_to) => {
                        return Err(InvalidEmissionOwner(header.bind_to));
                    },
                    NewUTXO(index) => {
                        let tx_committing_to_contract = utxos_spent_in.get(&self.contract.header.issuance_utxo);
                        if tx_committing_to_contract == None {
                            return Err(MissingTx(header.bind_to));
                        }
                        let tx_committing_to_contract = tx_committing_to_contract.unwrap();

                        if !tx_committing_to_contract.is_input(&self.contract.header.issuance_utxo) {
                            return Err(WrongOutputSpentEntry(self.contract.header.issuance_utxo));
                        }

                        let txoutref = TxOutRef::from_tx(tx_committing_to_contract, index as usize);
                        let txoutref = txoutref.unwrap();

                        if txoutref != header.bind_to {
                            return Err(InvalidEmissionOwner(header.bind_to));
                        }
                    },
                    _ => {}
                }

                // Add the only virtual input
                let mut inputs: HashMap<Sha256dHash, u64> = HashMap::new();
                inputs.insert(self.contract.token_id(), self.contract.header.total_supply as u64);

                let mut outputs: HashMap<Sha256dHash, u64> = HashMap::new();
                for output in header.outputs.iter() {
                    *outputs.entry(output.token_id).or_insert(0) += output.amount as u64;
                }

                // Make sure the outputs matches the amount of the virtual input
                if inputs == outputs {
                    Ok(())
                } else {
                    Err(InputOutputMismatch)
                }
            }
            _ => {
                Err(WrongContractKind(self.contract.data.clone()))
            }
        }
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for EmissionContractProofData {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Add the kind of proof
        2u8.consensus_encode(s)?;

        // Encode the contract
        self.contract.consensus_encode(s)?;

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for EmissionContractProofData {
    fn consensus_decode(d: &mut D) -> Result<EmissionContractProofData, D::Error> {
        Ok(EmissionContractProofData {
            contract: ConsensusDecodable::consensus_decode(d)?
        })
    }
}