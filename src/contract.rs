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

//! RGB contracts
//!
//! Implementation of data structures used in RGB contracts

use std::io::Cursor;

use bitcoin_hashes::{sha256d, Hash};
use bitcoin::OutPoint;
use bitcoin::consensus::encode::*;
use bitcoin::network::constants::Network;
use secp256k1::PublicKey;

use crate::{IdentityHash, RgbError, OnChain};

/// Commitment scheme variants used by RGB contract header field `commitment_scheme`.
/// With the current specification only two possible schemes are supported: OP_RETURN and
/// pay-to-contract. See more at <https://github.com/rgb-org/spec/blob/master/01-rgb.md#commitment-scheme>
///
/// NB: Commitment scheme specifies the way of commiting proofs for RGB transactions, not
/// the way by which original RGB contract is commited
#[repr(u8)]
#[derive(Clone, Debug)]
pub enum CommitmentScheme {
    /// Used by reissuance blueprint contract, which inherits `commitment_scheme` from
    /// the original issuance contract.
    NotApplicable,

    /// OP_RETURN scheme, committing RGB proofs to a special bitcoin transaction output
    /// containing `OP_RETURN` opcode followed by the hash of RGB contract or proof
    OpReturn,

    /// Pay to contract scheme, committing RGB proofs to a bitcoin UTXO via public key tweak.
    PayToContract,
}

impl From<u8> for CommitmentScheme {
    fn from(no: u8) -> Self {
        match no {
            0x1 => CommitmentScheme::OpReturn,
            0x2 => CommitmentScheme::PayToContract,
            _ => CommitmentScheme::NotApplicable,
        }
    }
}

impl From<CommitmentScheme> for u8 {
    fn from(scheme: CommitmentScheme) -> Self {
        match scheme {
            CommitmentScheme::OpReturn => 0x1,
            CommitmentScheme::PayToContract => 0x2,
            CommitmentScheme::NotApplicable => 0x0,
        }
    }
}


/// Types of blueprints for the RGB contracts. Each blueprint type defines custom fields used
/// in the contract body – and sometimes special requirements for the contract header fields.
/// Read more on <https://github.com/rgb-org/spec/blob/master/01-rgb.md#blueprints-and-versioning>
///
/// Subjected to the future extension, at this moment this is very preliminary work.
#[repr(u16)]
#[derive(Clone, Debug)]
pub enum BlueprintType {
    /// Simple issuance contract
    Issue,

    /// Crowdsale contract
    Crowdsale,

    /// Reissuing contract
    Reissue,

    /// Reserved for all other blueprints which are unknown for the current version
    Unknown,
}

impl From<u16> for BlueprintType {
    fn from(no: u16) -> Self {
        match no {
            0x0001 => BlueprintType::Issue,
            0x0002 => BlueprintType::Crowdsale,
            0x0003 => BlueprintType::Reissue,
            _ => BlueprintType::Unknown,
        }
    }
}


impl From<BlueprintType> for u16 {
    fn from(blueprint: BlueprintType) -> Self {
        match blueprint {
            BlueprintType::Issue => 0x0001,
            BlueprintType::Crowdsale => 0x0002,
            BlueprintType::Reissue => 0x0003,
            BlueprintType::Unknown => 0xFFFF,
        }
    }
}

/// Contract header fields required by the specification
#[derive(Clone, Debug)]
pub struct ContractHeader {
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

impl<S: Encoder> Encodable<S> for ContractHeader {
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

impl<D: Decoder> Decodable<D> for ContractHeader {
    fn consensus_decode(d: &mut D) -> Result<ContractHeader, Error> {
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

/// Trait to be used by custom contract blueprint implementation to provide its own custom fields.
pub trait ContractBody: Sized { }


/// Simple issuance contract
///
/// **Version 0x0008**
/// This blueprint allows to mint `total_supply` tokens and immediately send them
/// to the `owner_utxo`.
#[derive(Clone, Debug)]
pub struct IssuanceContractBody {
    /// UTXO which will receive all the tokens
    pub owner_utxo: OutPoint,
}

impl ContractBody for IssuanceContractBody {

}

impl<S: Encoder> Encodable<S> for IssuanceContractBody {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.owner_utxo.consensus_encode(s)
    }
}
impl<D: Decoder> Decodable<D> for IssuanceContractBody {
    fn consensus_decode(d: &mut D) -> Result<IssuanceContractBody, Error> {
        Ok(IssuanceContractBody {
            owner_utxo: Decodable::consensus_decode(d)?
        })
    }
}

/// Crowdsale contract
///
/// This blueprint allows to set-up a crowdsale, to sell tokens at a specified price up to the
/// `total_supply`. This contract actually creates two different assets with different
/// `assets_id`s. Together with the "normal" token, a new "change" token is issued,
/// to "refund" users who either send some Bitcoins too early or too late and will miss out
/// on the crowdsale. Change tokens have a fixed 1-to-1-satoshi rate in the issuing phase,
/// and are intended to maintain the same rate in the redeeming phase.
///
/// **Version 0x0008**
/// The additional fields in the body are:
/// * `deposit_address`: an address to send Bitcoins to in order to buy tokens
/// * `price_sat`: a price in satoshi for a single token
/// * `from_block`: block height after which crowdsale ends
/// * `to_block`: block height at which crowdsale starts
#[derive(Clone, Debug)]
pub struct CrowdsaleContractBody {
    // FIXME: It's unclear how two different asset types are supported by this contract
    // and how their `asset_id`s are defined.
    // For more details see issue #72 <https://github.com/rgb-org/spec/issues/72>

    /// An address to send Bitcoins to in order to buy tokens
    pub deposit_address: String,

    /// A price (in satoshi) for a single token
    pub price_sat: u64,

    /// Block height at which crowdsale starts
    pub from_block: u64,

    /// Block height after which crowdsale ends
    pub to_block: u64,
}

impl ContractBody for CrowdsaleContractBody {

}

impl<S: Encoder> Encodable<S> for CrowdsaleContractBody {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.deposit_address.consensus_encode(s)?;
        self.price_sat.consensus_encode(s)?;
        self.from_block.consensus_encode(s)?;
        self.to_block.consensus_encode(s)
    }
}
impl<D: Decoder> Decodable<D> for CrowdsaleContractBody {
    fn consensus_decode(d: &mut D) -> Result<CrowdsaleContractBody, Error> {
        let deposit_address: String = Decodable::consensus_decode(d)?;
        let price_sat: u64 = Decodable::consensus_decode(d)?;
        let from_block: u64 = Decodable::consensus_decode(d)?;
        let to_block: u64 = Decodable::consensus_decode(d)?;

        Ok(CrowdsaleContractBody {
            deposit_address,
            price_sat,
            from_block,
            to_block
        })
    }
}

/// Reissuing contract
///
/// This blueprint allows an asset issuer to re-issue tokens by inflating the supply.
/// This is allowed only if the original contract had `reissuance_enabled` != 0.
///
/// This contract MUST be issued using the `reissuance_utxo` and its version MUST match
/// the original contract's one.
///
/// **Version 0x0008**
/// The following fields in its header MUST be set to 0 (for integer values) or empty-length
/// strings in order to disable them:
/// * `title`
/// * `description`
/// * `network`
/// * `min_amount`
/// * `max_hops`
/// * `burn_address`
/// * `commitment_scheme`
///
/// The following fields MUST be filled with "real" values:
/// * `contract_url`: Unique url for the publication of the contract and the light-anchors
/// * `issuance_utxo`: The UTXO which will be spent in a transaction containing a commitment
///    to this contract to "deploy" it (must match the original contract's `reissuance_utxo`)
/// * `total_supply`: Additional supply in satoshi (1e-8)
/// * `reissuance_enabled`: Whether the re-issuance feature is enabled or not
/// * `reissuance_utxo`: (optional) UTXO which have to be spent to reissue tokens
/// * `version`: 16-bit number representing version of the blueprint used
///
/// There are no additional fields in its body.
#[derive(Clone, Debug)]
pub struct ReissueContractBody {
}

impl ContractBody for ReissueContractBody {
}


impl<S: Encoder> Encodable<S> for ReissueContractBody {
    fn consensus_encode(&self, _: &mut S) -> Result<(), Error> {
        Ok(())
    }
}
impl<D: Decoder> Decodable<D> for ReissueContractBody {
    fn consensus_decode(_: &mut D) -> Result<ReissueContractBody, Error> {
        Ok(ReissueContractBody { })
    }
}

/// RGB Contract in-memory representation.
///
/// Data structure provides serialization with consensus serialization methods
/// for disk storage and network messaging between Bifröst servers and RGB-enabled wallets,
/// verification of the contract internal consistency and blueprint specification
/// and tool methods for generating bitcoin output scripts for the associated on-chain transaction.
#[derive(Clone, Debug)]
pub struct Contract<B: ContractBody> {
    /// Contract header, containing fixed set of fields, shared by all contract blueprints
    pub header: ContractHeader,

    /// Contract body, with blueprint-specific set of fields
    pub body: Box<B>,

    /// Original public key used for signing the contract. Used for pay-to-contract schemes only.
    /// Serialized, but not a part of the commitment hash.
    pub original_commitment_pk: Option<PublicKey>,
}

impl<B: ContractBody> OnChain<B> for Contract<B> where B: Encodable<Cursor<Vec<u8>>> {
    /// Provides unique asset_id, which is computed as a SHA256d-hash from the consensus-serialized
    /// contract data
    fn get_identity_hash(&self) -> IdentityHash {
        let hash = serialize(self);
        sha256d::Hash::from_slice(hash.as_slice()).unwrap()
    }

    /// Returns RGB contract, i.e. itself
    fn get_contract(&self) -> Result<&Contract<B>, RgbError<B>> {
        Ok(&self)
    }

    /// Returns untweaked public key if the pay-to-contract commitment scheme is used.
    fn get_original_pk(&self) -> Option<PublicKey> {
        self.original_commitment_pk
    }
}

impl<S: Encoder, T: Encodable<S> + ContractBody> Encodable<S> for Contract<T> {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.header.consensus_encode(s)?;
        (*self.body).consensus_encode(s)?;

        // We do not need to serialize a flag whether `original_commitment_pk` is present since
        // its presence is defined by the `commitment_scheme` field in the contract header,
        // which is already serialized
        match self.original_commitment_pk {
            Some(pk) => pk.serialize().consensus_encode(s),
            None => Ok(()),
        }
    }
}

impl<D: Decoder, T: Decodable<D> + ContractBody> Decodable<D> for Contract<T> {
    fn consensus_decode(d: &mut D) -> Result<Contract<T>, Error> {
        let header: ContractHeader = Decodable::consensus_decode(d)?;
        let body: Box<T> = Box::new(Decodable::consensus_decode(d)?);
        let mut original_commitment_pk: Option<PublicKey> = None;
        match header.commitment_scheme {
            CommitmentScheme::PayToContract => {
                let data: Vec<u8> = Decodable::consensus_decode(d)?;
                match PublicKey::from_slice(&data[..]) {
                    Ok(pk) => original_commitment_pk = Some(pk),
                    Err(_) => return Err(
                        bitcoin::consensus::encode::Error::ParseFailed("Can't decode public key"))
                };
            },
            _ => ()
        };

        Ok(Contract{ header, body, original_commitment_pk })
    }
}
