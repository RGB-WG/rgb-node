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

use crate::*;

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
    // and how their `get_identity_hash`s are defined.
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

impl ContractBody for CrowdsaleContractBody {}

impl Verify<Self> for CrowdsaleContractBody {
    // TODO: Do the actual verification for ReissueContractBody instead of the default empty one
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
            to_block,
        })
    }
}
