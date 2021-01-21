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
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::OutPoint;
use lnpbp::seals::OutpointReveal;
use rgb::{Consignment, ContractId};
use rgb20::{ConsealCoins, OutpointCoins, SealCoins};

use crate::DataFormat;

#[derive(Clone, Debug, Display, LnpApi)]
#[encoding_crate(lnpbp::strict_encoding)]
#[lnp_api(encoding = "strict")]
#[display(Debug)]
#[non_exhaustive]
pub enum Request {
    #[lnp_api(type = 0x0101)]
    Issue(crate::rpc::fungible::Issue),

    #[lnp_api(type = 0x0103)]
    Transfer(crate::rpc::fungible::TransferApi),

    #[lnp_api(type = 0x0105)]
    Validate(::rgb::Consignment),

    #[lnp_api(type = 0x0107)]
    Accept(crate::rpc::fungible::AcceptApi),

    #[lnp_api(type = 0x0109)]
    ImportAsset(::rgb::Genesis),

    #[lnp_api(type = 0x010b)]
    ExportAsset(::rgb::ContractId),

    #[lnp_api(type = 0x010d)]
    Forget(::bitcoin::OutPoint),

    #[lnp_api(type = 0xFF01)]
    Sync(DataFormat),

    #[lnp_api(type = 0xFF02)]
    Assets(OutPoint),

    #[lnp_api(type = 0xFF03)]
    Allocations(ContractId),
}

#[derive(
    Clap, Clone, PartialEq, StrictEncode, StrictDecode, Debug, Display,
)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize,),
    serde(crate = "serde_crate")
)]
pub struct Issue {
    /// Asset ticker (up to 8 characters, always converted to uppercase)
    #[clap(validator=ticker_validator)]
    pub ticker: String,

    /// Asset name (up to 32 characters)
    pub name: String,

    /// Asset description
    #[clap(short, long)]
    pub description: Option<String>,

    /// Precision, i.e. number of digits reserved for fractional part
    #[clap(short, long, default_value = "0")]
    pub precision: u8,

    /// Asset allocation, in form of <amount>@<txid>:<vout>
    pub allocation: Vec<OutpointCoins>,

    /// Outputs controlling inflation (secondary issue);
    /// in form of <amount>@<txid>:<vout>
    #[clap(short, long)]
    pub inflation: Vec<OutpointCoins>,

    /// Enable renomination procedure; parameter takes argument in form of
    /// <txid>:<vout> specifying output controlling renomination right
    #[clap(short, long)]
    pub renomination: Option<OutPoint>,

    /// Enable epoch-based burn & replacement procedure; parameter takes
    /// argument in form of <txid>:<vout> specifying output controlling the
    /// right of opening the first epoch
    #[clap(short, long)]
    pub epoch: Option<OutPoint>,
}

#[derive(Clone, PartialEq, StrictEncode, StrictDecode, Debug, Display)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display(Debug)]
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
    pub ours: Vec<SealCoins>,

    /// Receiver's allocations.
    ///
    /// They are kept separate from change allocations since here we do not
    /// know the actual seals and only know hashes derived from seal data and
    /// blinding entropy.
    pub theirs: Vec<ConsealCoins>,
}

#[derive(Clone, StrictEncode, StrictDecode, Debug, Display)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display(Debug)]
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
            "Ticker name must be between 3 and 8 chars, contain no spaces and \
            consist only of capital letters\
            "
            .to_string(),
        )
    } else {
        Ok(())
    }
}
