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

use bitcoin::consensus::encode::*;
use bitcoin::OutPoint;

use crate::*;

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

impl ContractBody for IssuanceContractBody {}

impl Verify<Self> for IssuanceContractBody {
    // Nothing to check here, so we default to the trait implementation always returning `Ok`
}

impl<S: Encoder> Encodable<S> for IssuanceContractBody {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        self.owner_utxo.consensus_encode(s)
    }
}
impl<D: Decoder> Decodable<D> for IssuanceContractBody {
    fn consensus_decode(d: &mut D) -> Result<IssuanceContractBody, Error> {
        Ok(IssuanceContractBody {
            owner_utxo: Decodable::consensus_decode(d)?,
        })
    }
}
