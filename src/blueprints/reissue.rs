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

use bitcoin::consensus::encode::*;

use crate::*;

/// Reissuing contract
///
/// This blueprint allows an asset issuer to re-issue tokens by inflating the supply.
/// This is allowed only if the original contract had `reissuance_enabled` != 0.
///
/// This contract MUST be issued using the `reissuance_utxo` and its version MUST match
/// the original contract's one.
///
/// **Version 0x0008**
/// There are no additional fields in its body.
#[derive(Clone, Debug)]
pub struct ReissueContractBody {
    /// Weak reference to the contract itself – we need it for the verification process
    pub contract: Weak<Contract<Self>>,
}

impl ContractBody for ReissueContractBody { }

impl Verify<Self> for ReissueContractBody {
    /// Function performing verification of the integrity for the RGB contract body.
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
    ///
    /// # Arguments:
    /// * `tx_provider` - a specially-formed callback function provided by the callee (wallet app
    /// or bifrost server) that returns transaction for a given case (specified by `TxQuery`-typed
    /// argument given to the callback). Used during the verification process to check on-chain
    /// part of the contract. Since rgblib has no direct access to a bitcoin node
    /// (it's rather a task for particular wallet or Bifrost implementation) it relies on this
    /// callback during the verification process.
    fn verify(&self, _tx_provider: TxProvider<Self>) -> Result<(), RgbError<Self>> {
        let contract = self.contract.upgrade().unwrap();
        let header = &contract.header;

        // FIXME: We need Network to be set to the `None` value, however it is not supported.
        match (header.title.len(), &header.description, header.network, header.min_amount,
              header.max_hops, &header.burn_address, &header.commitment_scheme) {
            (0, None, _, 0, None, None, CommitmentScheme::NotApplicable) => (),
            _ => return Err(RgbError::InternalContractIncosistency(contract,
               "Contract header fields have inappropriate value for the Reissuing contract blueprint"
            )),
        }

        // TODO: Complete verification of the blueprint

        Ok(())
    }
}

impl<S: Encoder> Encodable<S> for ReissueContractBody {
    fn consensus_encode(&self, _: &mut S) -> Result<(), Error> {
        Ok(())
    }
}
impl<D: Decoder> Decodable<D> for ReissueContractBody {
    fn consensus_decode(_: &mut D) -> Result<ReissueContractBody, Error> {
        Ok(ReissueContractBody { contract: Weak::new() })
    }
}

