use bitcoin::BitcoinHash;
use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::script::Script;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::network::serialize;
use bitcoin::network::serialize::serialize;
use bitcoin::network::serialize::SimpleDecoder;
use bitcoin::network::serialize::SimpleEncoder;
use bitcoin::Transaction;
use bitcoin::util::address::Address;
use bitcoin::util::hash::Hash160;
use bitcoin::util::hash::Sha256dHash;
use pay_to_contract::ECTweakFactor;
use secp256k1::Error;
use secp256k1::PublicKey;
use secp256k1::Secp256k1;
use std::collections::HashMap;
use super::bitcoin::network::constants::Network;
use super::bitcoin::OutPoint;
use super::traits::Verify;
use traits::NeededTx;
use traits::PayToContract;

#[derive(Clone, Debug)]
pub struct Contract {
    pub title: String,
    pub issuance_utxo: OutPoint,
    pub initial_owner_utxo: OutPoint,
    pub network: Network,
    pub total_supply: u64,
    pub original_commitment_pk: Option<PublicKey>
}

impl Contract {
    pub fn get_asset_id(&self) -> Sha256dHash {
        self.bitcoin_hash()
    }
}

impl BitcoinHash for Contract {
    fn bitcoin_hash(&self) -> Sha256dHash { // all the fields
        // TODO: leave out "original_commitment_pk": it's not necessary to "pre-commit" to this value,
        // and doing so could actually make some tokens/bitcoins unspendable (if original_commitment_pk is not set)
        Sha256dHash::from_data(&serialize(self).unwrap())
    }
}

impl Verify for Contract {
    fn get_needed_txs(&self) -> Vec<NeededTx> {
        vec![NeededTx::WhichSpendsOutPoint(self.issuance_utxo)]
    }

    fn verify(&self, needed_txs: &HashMap<&NeededTx, Transaction>) -> bool {
        let committing_tx = needed_txs.get(&NeededTx::WhichSpendsOutPoint(self.issuance_utxo)).unwrap();

        // TODO: signal the commitment output somehow
        if committing_tx.output[0].script_pubkey != self.get_expected_script() {
            println!("invalid commitment");
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

impl PayToContract for Contract {
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

impl<S: SimpleEncoder> ConsensusEncodable<S> for Contract {
    fn consensus_encode(&self, s: &mut S) -> Result<(), serialize::Error> {
        self.title.consensus_encode(s)?;
        self.issuance_utxo.consensus_encode(s)?;
        self.initial_owner_utxo.consensus_encode(s)?;

        self.network.consensus_encode(s)?;
        self.total_supply.consensus_encode(s)?;

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

impl<D: SimpleDecoder> ConsensusDecodable<D> for Contract {
    fn consensus_decode(d: &mut D) -> Result<Contract, serialize::Error> {
        let title: String = ConsensusDecodable::consensus_decode(d)?;
        let issuance_utxo: OutPoint = ConsensusDecodable::consensus_decode(d)?;
        let initial_owner_utxo: OutPoint = ConsensusDecodable::consensus_decode(d)?;

        let mut c = Contract {
            title,
            issuance_utxo,
            initial_owner_utxo,
            network: ConsensusDecodable::consensus_decode(d)?,
            total_supply: ConsensusDecodable::consensus_decode(d)?,
            original_commitment_pk: None
        };

        let original_commitment_pk_ser: Vec<u8> = ConsensusDecodable::consensus_decode(d)?;
        if original_commitment_pk_ser.len() > 0 {
            let s = Secp256k1::new();
            c.set_commitment_pk(&PublicKey::from_slice(&s, &original_commitment_pk_ser.as_slice()).unwrap());
        }

        Ok(c)
    }
}