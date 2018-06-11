use entities::proofs::dummy_proof::DummyProofData;
use entities::proofs::emission_contract_proof::EmissionContractProofData;
use entities::proofs::transfer_proof::TransferProofData;
use entities::traits::VerifyData;
use std::collections::HashMap;
use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::blockdata::transaction::Transaction;
use entities::proofs::header::ProofHeader;
use entities::proofs::data::ProofData::BranchDummyProof;
use entities::proofs::data::ProofData::BranchTranferProof;
use entities::proofs::data::ProofData::BranchEmissionContractProof;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::encodable::ConsensusDecodable;
use util;

#[derive(Clone, Debug)]
pub enum ProofData {
    BranchDummyProof(DummyProofData),
    BranchEmissionContractProof(EmissionContractProofData),
    BranchTranferProof(TransferProofData),
}

impl VerifyData for ProofData {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>, header: &ProofHeader) -> Result<(), util::error::Error> {
        match *self {
            BranchDummyProof(ref data) => data.verify(utxos_spent_in, header),
            BranchTranferProof(ref data) => data.verify(utxos_spent_in, header),
            BranchEmissionContractProof(ref data) => data.verify(utxos_spent_in, header),
        }
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for ProofData {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            BranchDummyProof(ref data) => data.consensus_encode(s),
            BranchTranferProof(ref data) => data.consensus_encode(s),
            BranchEmissionContractProof(ref data) => data.consensus_encode(s),
        }
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for ProofData {
    fn consensus_decode(d: &mut D) -> Result<ProofData, D::Error> {
        let proof_data_type: u8 = ConsensusDecodable::consensus_decode(d)?;

        match proof_data_type {
            0x01 => {
                let proof_data: TransferProofData = ConsensusDecodable::consensus_decode(d)?;
                Ok(ProofData::BranchTranferProof(proof_data))
            }
            0x02 => {
                let proof_data: EmissionContractProofData = ConsensusDecodable::consensus_decode(d)?;
                Ok(ProofData::BranchEmissionContractProof(proof_data))
            }
            0xFF => {
                let proof_data: DummyProofData = ConsensusDecodable::consensus_decode(d)?;
                Ok(ProofData::BranchDummyProof(proof_data))
            }
            x => Err(d.error(format!("ProofData type {:02x} not understood", x)))
        }
    }
}