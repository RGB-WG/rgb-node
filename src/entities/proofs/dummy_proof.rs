use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize::SimpleEncoder;
use entities::proofs::Proof;
use entities::proofs::header::ProofHeader;
use entities::rgb_output::RgbOutput;
use entities::traits::VerifyData;
use std::collections::HashMap;
use util;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::encodable::ConsensusDecodable;
use entities::proofs::data::ProofData::BranchDummyProof;

#[derive(Clone, Debug)]
pub struct DummyProofData {}

impl DummyProofData {
    pub fn new_data() -> DummyProofData {
        DummyProofData {}
    }

    pub fn new(bind_to: TxOutRef, inputs: &[Proof], outputs: &[RgbOutput]) -> Proof {
        Proof::new(
            ProofHeader::new(bind_to, inputs, outputs),
            BranchDummyProof(DummyProofData::new_data()),
        )
    }
}

impl VerifyData for DummyProofData {
    fn verify(&self, _utxos_spent_in: &HashMap<&TxOutRef, Transaction>, _header: &ProofHeader) -> Result<(), util::error::Error> {
        Ok(())
    }
}

impl<S: SimpleEncoder> ConsensusEncodable<S> for DummyProofData {
    fn consensus_encode(&self, s: &mut S) -> Result<(), S::Error> {
        // Add the kind of proof
        255u8.consensus_encode(s)?;

        Ok(())
    }
}

impl<D: SimpleDecoder> ConsensusDecodable<D> for DummyProofData {
    fn consensus_decode(d: &mut D) -> Result<DummyProofData, D::Error> {
        Ok(DummyProofData {})
    }
}