// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use regex::Regex;
use serde::{Deserialize, Serialize};

use lnpbp::bitcoin::util::psbt::PartiallySignedTransaction;
use lnpbp::bitcoin::OutPoint;
use lnpbp::bp::blind::OutpointReveal;
use lnpbp::rgb::{Amount, Consignment, ContractId};

use crate::fungible::{Outcoincealed, Outcoins};
use crate::util::SealSpec;

#[derive(Clone, Debug, Display, LnpApi)]
#[lnp_api(encoding = "strict")]
#[display_from(Debug)]
#[non_exhaustive]
pub enum Request {
    #[lnp_api(type = 0x0101)]
    Issue(crate::api::fungible::Issue),

    #[lnp_api(type = 0x0103)]
    Transfer(crate::api::fungible::TransferApi),

    #[lnp_api(type = 0x0105)]
    Accept(crate::api::fungible::AcceptApi),

    #[lnp_api(type = 0x0107)]
    ImportAsset(::lnpbp::rgb::Genesis),

    #[lnp_api(type = 0x0109)]
    ExportAsset(::lnpbp::rgb::ContractId),

    #[lnp_api(type = 0xFF01)]
    Sync,
}

#[derive(
    Clap, Clone, PartialEq, Serialize, Deserialize, StrictEncode, StrictDecode, Debug, Display,
)]
#[display_from(Debug)]
pub struct Issue {
    /// Asset ticker
    #[clap(validator=ticker_validator)]
    pub ticker: String,

    /// Asset title
    pub title: String,

    /// Asset description
    #[clap(short, long)]
    pub description: Option<String>,

    /// Limit for the total supply; ignored if the asset can't be inflated
    #[clap(short, long)]
    pub supply: Option<f32>,

    /// Enables secondary issuance/inflation; takes UTXO seal definition
    /// as its value
    #[clap(short, long, requires("supply"))]
    pub inflatable: Option<SealSpec>,

    /// Precision, i.e. number of digits reserved for fractional part
    #[clap(short, long, default_value = "0")]
    pub precision: u8,

    /// Dust limit for asset transfers; defaults to no limit
    #[clap(short = "D", long)]
    pub dust_limit: Option<Amount>,

    /// Asset allocation, in form of <amount>@<txid>:<vout>
    #[clap(required = true)]
    pub allocate: Vec<Outcoins>,
}

#[derive(Clone, PartialEq, StrictEncode, StrictDecode, Debug, Display)]
#[display_from(Debug)]
pub struct TransferApi {
    /// Asset contract id
    pub contract_id: ContractId,

    /// Base layer transaction structure to use
    pub psbt: PartiallySignedTransaction,

    /// Asset input: unspent transaction outputs
    pub inputs: Vec<OutPoint>,

    /// Asset change allocations
    ///
    /// Here we always know an explicit outpoint that will contain the assets
    pub ours: Vec<Outcoins>,

    /// Receiver's allocations.
    ///
    /// They are kept separate from change allocations since here we do not
    /// know the actual seals and only know hashes derived from seal data and
    /// blinding entropy.
    pub theirs: Vec<Outcoincealed>,

    /// Optional change output: the rest of assets will be allocated here
    pub change: OutPoint,
}

#[derive(Clone, StrictEncode, StrictDecode, Debug, Display)]
#[display_from(Debug)]
pub struct AcceptApi {
    /// Raw consignment data
    pub consignment: Consignment,

    /// Reveal outpoints data used during invoice creation
    pub reveal_outpoints: Vec<OutpointReveal>,
}

fn ticker_validator(name: &str) -> Result<(), String> {
    let re = Regex::new(r"^[A-Z]{3,8}$").expect("Regex parse failure");
    if !re.is_match(&name) {
        Err(
            "Ticker name must be between 2 and 8 chars, contain no spaces and \
            consist only of capital letters\
            "
            .to_string(),
        )
    } else {
        Ok(())
    }
}
