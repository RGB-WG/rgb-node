use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleEncoder;
use entities::proofs::dummy_proof::DummyProofData;
use entities::proofs::emission_contract_proof::EmissionContractProofData;
use entities::proofs::ProofData::{BranchDummyProof, BranchTranferProof};
use entities::proofs::ProofData::BranchEmissionContractProof;
use entities::proofs::transfer_proof::TransferProofData;
use entities::rgb_output::RgbOutput;
use entities::traits::ContainsSignatures;
use entities::traits::Verify;
use entities::traits::VerifyData;
use std::collections::{HashMap, HashSet};
use util;
use util::error::Error::MissingTx;
use util::error::Error::LoopDetected;
use bitcoin::network::encodable::VarInt;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::util::hash::Sha256dHash;
use entities::proofs::header::ProofHeader;
use entities::proofs::data::ProofData;

pub mod dummy_proof;
pub mod emission_contract_proof;
pub mod transfer_proof;

pub mod header;
pub mod data;

#[derive(Clone, Debug)]
pub struct Proof {
    pub header: ProofHeader,
    pub data: ProofData,
}

impl Proof {
    pub fn new(header: ProofHeader, data: ProofData) -> Proof {
        Proof {
            header,
            data,
        }
    }
}

impl Verify for Proof {
    fn verify(&self, utxos_spent_in: &HashMap<&TxOutRef, Transaction>) -> Result<(), util::error::Error> {
        Ok(())
            .and_then(|_: ()| { self.header.verify(utxos_spent_in) })
            .and_then(|_: ()| { self.data.verify(utxos_spent_in, &self.header) })
    }

    fn get_necessary_txs(&self, set: &mut HashSet<TxOutRef>) {
        self.header.get_necessary_txs(set);
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for Proof {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        self.header.consensus_encode(s)?;
        self.data.consensus_encode(s)?;

        self.header.inputs.consensus_encode(s)?;

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for Proof {
    fn consensus_decode(d: &mut D) -> Result<Proof, D::Error> {
        let mut proof = Proof {
            header: ConsensusDecodable::consensus_decode(d)?,
            data: ConsensusDecodable::consensus_decode(d)?,
        };

        let mut inputs: Vec<Proof> = ConsensusDecodable::consensus_decode(d)?;

        proof.header.inputs.append(&mut inputs);

        Ok(proof)
    }
}