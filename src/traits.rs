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

use std::io::Write;

use bitcoin_hashes::{sha256d, Hash};
use bitcoin::{Transaction, OutPoint};
use bitcoin::blockdata::{opcodes::all::*, script::Builder, script::Script};
use secp256k1::{Secp256k1, PublicKey};

use crate::*;
use bitcoin::util::contracthash::tweak_key;
use bitcoin::util::psbt::serialize::Serialize;

pub enum TxQuery {
    TxId(sha256d::Hash),
    SpendingTxId(sha256d::Hash),
    SpendingVout(OutPoint),
}

pub type TxProvider<B> = fn(tx_type: TxQuery) -> Result<Transaction, RgbError<'static, B>>;

/// Trait describing and implementing shared functionality for the on-chain parts of the
/// RGB contracts and prootfs
pub trait OnChain<B: ContractBody> {
    fn get_identity_hash(&self) -> IdentityHash;
    fn get_contract(&self) -> Result<&Contract<B>, RgbError<B>>;
    fn get_original_pk(&self) -> Option<PublicKey>;

    /// Generates Bitcoin script corresponding to the given RGB proof (taking into account
    /// commitment scheme of the source RGB contract)
    fn get_script(&self) -> Result<Script, RgbError<B>> {
        let commitment_scheme = self.get_contract()?.header.commitment_scheme.clone();
        let builder = match commitment_scheme {
            CommitmentScheme::OpReturn => Builder::new()
                // Simple OP_RETURN with data corresponding to the hash of the current proof
                .push_opcode(OP_RETURN)
                .push_slice(&self.get_identity_hash().into_inner()),

            CommitmentScheme::PayToContract => Builder::new()
                // Pay to contract: standard P2PKH utilizing tweaked public key
                .push_opcode(OP_DUP)
                .push_opcode(OP_HASH160)
                // Pushing the hash of the tweaked public key
                .push_slice(
                    &sha256d::Hash::from_engine({
                        let mut engine = sha256d::Hash::engine();
                        engine.write_all(
                            // Tweaking public key
                            &tweak_key(
                                &Secp256k1::new(),
                                bitcoin::PublicKey {
                                    compressed: true,
                                    key: self.get_original_pk().ok_or_else(
                                        || RgbError::NoOriginalPubKey(self.get_identity_hash())
                                    )?
                                },
                                &[
                                    // Adding "RGB" string according to
                                    // <https://github.com/rgb-org/spec/issues/61>
                                    "RGB".as_bytes(),
                                    // Adding contract hash
                                    &self.get_identity_hash()[..]
                                ].concat()[..]
                            ).serialize()
                        )?;
                        engine
                    }
                    )[..]
                )
                .push_opcode(OP_EQUALVERIFY)
                .push_opcode(OP_CHECKSIG),

            _ => return Err(RgbError::UnsupportedCommitmentScheme(commitment_scheme)),
        };
        Ok(builder.into_script())
    }
}

/// Trait for verifiable entries
pub trait Verify<B: ContractBody> {
    /// Function performing verification of the integrity for the RGB entity (contract or particular
    /// proof) for both on-chain and off-chain parts; including internal consistency, integrity,
    /// proper formation of commitment transactions etc. This is default implementation,
    /// it checks nothing
    ///
    /// # Arguments:
    /// * `tx_provider` - a specially-formed callback function provided by the callee (wallet app
    /// or bifrost server) that returns transaction for a given case (specified by `TxQuery`-typed
    /// argument given to the callback). Used during the verification process to check on-chain
    /// part of the proofs and contracts. Since rgblib has no direct access to a bitcoin node
    /// (it's rather a task for particular wallet or Bifrost implementation) it relies on this
    /// callback during the verification process.
    fn verify(&self, _: TxProvider<B>) -> Result<(), RgbError<B>> {
        Ok(())
    }
}
