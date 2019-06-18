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
use bitcoin::blockdata::{opcodes::all::*, script::Builder, script::Script};
use secp256k1::PublicKey;

use crate::{IdentityHash, Contract, CommitmentScheme, RgbError};
use crate::contract::ContractBody;

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
                .push_slice(
                    &sha256d::Hash::from_engine({
                        let mut engine = sha256d::Hash::engine();
                        engine.write_all(
                            // TODO: Add actual public key tweaking
                            &self.get_original_pk()
                                .ok_or_else(|| RgbError::NoOriginalPubKey(self.get_identity_hash()))?
                                .serialize()
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
