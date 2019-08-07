// RGB Rust Library
// Written in 2019 by
//     Dr. Maxim Orlovsky <dr.orlovsky@gmail.com>
// basing on ideas from the original RGB rust library by
//     Alekos Filini <alekos.filini@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::rc::Weak;

use bitcoin::OutPoint;
use bitcoin::consensus::encode::*;
use bitcoin::network::constants::Network;

use crate::*;

/// Contract header fields required by the specification
#[derive(Clone, Debug)]
pub struct ContractHeader<B: ContractBody> {
    /// Weak reference to the contract itself
    pub contract: Weak<Contract<B>>,

    /// 16-bit unsigned integer representing version of the blueprint used
    pub version: u16,

    /// Title of the asset contract
    pub title: String,

    /// Description of the asset contract, optional
    pub description: Option<String>,

    /// Unique url for the publication of the contract and the light-anchors. Optional.
    pub contract_url: Option<String>,

    /// The UTXO which will be spent in a transaction containing a commitment
    /// to this contract to "deploy" it
    pub issuance_utxo: OutPoint,

    /// The Bitcoin network in use (mainnet, testnet)
    pub network: Network,

    /// Total supply in "satoshi" asset units (1e-8 of an issued asset)
    pub total_supply: u64,

    /// Minimum amount of tokens that can be transferred together, like a *dust limit*
    pub min_amount: u64,

    /// Maximum number of "hops" before the reissuance, optional (thus can be disabled)
    pub max_hops: Option<u32>,

    /// Whether the re-issuance feature is enabled or not
    pub reissuance_enabled: bool,

    /// UTXO which have to be spent to reissue tokens. Optional.
    pub reissuance_utxo: Option<OutPoint>,

    /// An address to send tokens to in order to burn them. Optional.
    pub burn_address: Option<String>,

    /// The commitment scheme used by this contract
    pub commitment_scheme: CommitmentScheme,

    /// Type of the blueprint specification that contract complies to
    pub blueprint_type: BlueprintType,
}

impl<B: ContractBody> ContractHeader<B> {
    /// Validates given proof to have a correct structure matching RGB contract header fields
    pub fn validate_proof<'a>(&self, proof: &'a Proof<B>) -> Result<(), RgbError<'a, B>>
        where Proof<B>: OnChain<B> {
        // Pay-to-contract proofs MUST have original public key (before applying tweak) specified,
        // the rest MUST NOT
        match (&self.commitment_scheme, proof.original_pubkey) {
            (CommitmentScheme::OpReturn, Some(_)) =>
                Err(RgbError::ProofStructureNotMatchingContract(proof)),
            (CommitmentScheme::PayToContract, None) =>
                Err(RgbError::NoOriginalPubKey(proof.get_identity_hash())),
            (CommitmentScheme::NotApplicable, _) =>
                Err(RgbError::UnsupportedCommitmentScheme(CommitmentScheme::NotApplicable)),
            _ => Ok(()),
        }
    }
}

impl<B: ContractBody> Verify<B> for ContractHeader<B> where Contract<B>: OnChain<B> {
    /// Function performing verification of the integrity for the RGB contract header for both
    /// on-chain and off-chain parts; including internal consistency, integrity, proper formation of
    /// commitment transactions etc.
    ///
    /// # Arguments:
    /// * `tx_provider` - a specially-formed callback function provided by the callee (wallet app
    /// or bifrost server) that returns transaction for a given case (specified by `TxQuery`-typed
    /// argument given to the callback). Used during the verification process to check on-chain
    /// part of the contract. Since rgblib has no direct access to a bitcoin node
    /// (it's rather a task for particular wallet or Bifrost implementation) it relies on this
    /// callback during the verification process.
    fn verify(&self, _tx_provider: TxProvider<B>) -> Result<(), RgbError<B>> {
        let contract = self.contract.upgrade().unwrap();

        // 1. Checking that the contract is of supported versions
        match self.version {
            0x0001 => return Err(RgbError::OutdatedContractVersion(contract)),
            0x0002 => (),
            _ => return Err(RgbError::UnknownContractVersion(contract)),
        }

        // 2. Checking for internal consistency
        // 2.1. We can't require minimum transaction amount to be larger than the total supply
        if self.min_amount > self.total_supply {
            return Err(RgbError::InternalContractIncosistency(contract,
                "The requirement for the minimum transaction amount exceeds total asset supply"
            ));
        }
        // 2.2. If we enable reissuance, we need to provide UTXO to spend the reissued tokens
        if self.reissuance_enabled && self.reissuance_utxo.is_none() {
            return Err(RgbError::InternalContractIncosistency(contract,
                "Asset reissuance is enabled, but no reissuance UTXO is provided"
            ));
        }

        Ok(())
    }
}

impl<B: ContractBody, S: Encoder> Encodable<S> for ContractHeader<B> {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.version.consensus_encode(s)?;
        self.title.consensus_encode(s)?;

        // For optional strings we use zero-length string to represent `None` value
        let zero: String = "".to_string();
        match self.description {
            Some(ref str) => str.consensus_encode(s)?,
            None => zero.consensus_encode(s)?,
        }
        match self.contract_url {
            Some(ref str) => str.consensus_encode(s)?,
            None => zero.consensus_encode(s)?,
        }

        self.issuance_utxo.consensus_encode(s)?;
        self.network.consensus_encode(s)?;
        self.total_supply.consensus_encode(s)?;
        self.min_amount.consensus_encode(s)?;

        // For optionals, we use first byte to determine presence of the value (0x0 for no value,
        // 0x1 for some value) and then, if there is a value presented, we deserialize it.
        match self.max_hops {
            Some(hops) => {
                true.consensus_encode(s)?;
                hops.consensus_encode(s)?;
            },
            None => false.consensus_encode(s)?,
        }
        self.reissuance_enabled.consensus_encode(s)?;
        match self.reissuance_utxo {
            Some(out) => {
                true.consensus_encode(s)?;
                out.consensus_encode(s)?;
            },
            None => false.consensus_encode(s)?,
        }
        match self.burn_address {
            Some(ref addr) => {
                true.consensus_encode(s)?;
                addr.consensus_encode(s)?;
            },
            None => false.consensus_encode(s)?,
        }
        let commitment_scheme_u: u8 = self.commitment_scheme.clone().into();
        commitment_scheme_u.consensus_encode(s)?;
        let ref blueprint_type_u: u16 = self.blueprint_type.clone().into();
        blueprint_type_u.consensus_encode(s)
    }
}

impl<B: ContractBody, D: Decoder> Decodable<D> for ContractHeader<B> {
    fn consensus_decode(d: &mut D) -> Result<ContractHeader<B>, Error> {
        let version: u16 = Decodable::consensus_decode(d)?;
        let title: String = Decodable::consensus_decode(d)?;

        // For optional strings we use zero-length string to represent `None` value
        let string: String = Decodable::consensus_decode(d)?;
        let description: Option<String> = match string.len() {
            0 => None,
            _ => Some(string),
        };
        let string: String = Decodable::consensus_decode(d)?;
        let contract_url: Option<String> = match string.len() {
            0 => None,
            _ => Some(string),
        };

        let issuance_utxo: OutPoint = Decodable::consensus_decode(d)?;
        let network: Network = Decodable::consensus_decode(d)?;
        let total_supply: u64 = Decodable::consensus_decode(d)?;
        let min_amount: u64 = Decodable::consensus_decode(d)?;

        // For optionals, we use first byte to determine presence of the value (0x0 for no value,
        // 0x1 for some value) and then, if there is a value presented, we deserialize it.
        let mut has_value: bool;

        let mut max_hops: Option<u32> = None;
        has_value = Decodable::consensus_decode(d)?;
        if has_value {
            max_hops = Some(Decodable::consensus_decode(d)?);
        }

        let reissuance_enabled: bool = Decodable::consensus_decode(d)?;

        let mut reissuance_utxo: Option<OutPoint> = None;
        has_value = Decodable::consensus_decode(d)?;
        if has_value {
            reissuance_utxo = Some(Decodable::consensus_decode(d)?);
        }

        let string: String = Decodable::consensus_decode(d)?;
        let burn_address: Option<String> = match string.len() {
            0 => None,
            _ => Some(string),
        };
        let commitment_scheme_id: u8 = Decodable::consensus_decode(d)?;
        let commitment_scheme= CommitmentScheme::from(commitment_scheme_id);
        let blueprint_type_id: u16 = Decodable::consensus_decode(d)?;
        let blueprint_type = BlueprintType::from(blueprint_type_id);

        Ok(ContractHeader {
            contract: Weak::new(),
            version,
            title,
            description,
            contract_url,
            issuance_utxo,
            network,
            total_supply,
            min_amount,
            max_hops,
            reissuance_enabled,
            reissuance_utxo,
            burn_address,
            commitment_scheme,
            blueprint_type
        })
    }
}
