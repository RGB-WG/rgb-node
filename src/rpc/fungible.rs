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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::OutPoint;
use bp::seals::OutpointReveal;
use rgb::{
    AtomicValue, Consignment, ContractId, Disclosure, Genesis, OutpointValue,
    SealDefinition, SealEndpoint,
};

use microservices::FileFormat;

#[derive(Clone, Debug, Display, Api)]
#[api(encoding = "strict")]
#[display(inner)]
#[non_exhaustive]
pub enum Request {
    #[api(type = 0x0101)]
    Issue(IssueReq),

    #[api(type = 0x0103)]
    Transfer(TransferReq),

    #[api(type = 0x0105)]
    #[display("validate(...)")]
    Validate(Consignment),

    #[api(type = 0x0107)]
    Accept(AcceptReq),

    #[api(type = 0x0108)]
    #[display("enclose({0})")]
    Enclose(Disclosure),

    #[api(type = 0x0109)]
    #[display("import_asset({0})")]
    ImportAsset(Genesis),

    #[api(type = 0x010b)]
    #[display("export_asset({0})")]
    ExportAsset(ContractId),

    #[api(type = 0x010d)]
    #[display("forget({0})")]
    Forget(OutPoint),

    #[api(type = 0xFF01)]
    #[display("sync(using: {0})")]
    Sync(FileFormat),

    #[api(type = 0xFF02)]
    #[display("assets(on: {0})")]
    Assets(OutPoint),

    #[api(type = 0xFF03)]
    #[display("allocations({0})")]
    Allocations(ContractId),
}

#[derive(
Parser, Clone, PartialEq, StrictEncode, StrictDecode, Debug, Display,
)]
#[display("issue({ticker}, {name}, precision: {precision}, ...)")]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize,),
    serde(crate = "serde_crate")
)]
pub struct IssueReq {
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
    pub allocation: Vec<OutpointValue>,

    /// Outputs controlling inflation (secondary issue);
    /// in form of <amount>@<txid>:<vout>
    #[clap(short, long)]
    pub inflation: Vec<OutpointValue>,

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
#[display("transfer({contract_id}, ...)")]
pub struct TransferReq {
    /// Asset contract id
    pub contract_id: ContractId,

    /// Base layer transaction structure to use
    pub witness: PartiallySignedTransaction,

    /// Asset input: unspent transaction outputs
    pub inputs: BTreeSet<OutPoint>,

    /// Receiver's allocations.
    ///
    /// They are kept separate from change allocations since here we do not
    /// know the actual seals and only know hashes derived from seal data and
    /// blinding entropy.
    pub payment: BTreeMap<SealEndpoint, AtomicValue>,

    /// Asset change allocations
    ///
    /// Here we always know an explicit outpoint that will contain the assets
    pub change: BTreeMap<SealDefinition, AtomicValue>,
}

#[derive(Clone, StrictEncode, StrictDecode, Debug, Display)]
#[display("accept(...)")]
pub struct AcceptReq {
    /// Raw consignment data
    pub consignment: Consignment,

    /// Reveal outpoints data used during invoice creation
    pub reveal_outpoints: Vec<OutpointReveal>,
}

fn ticker_validator(name: &str) -> Result<(), String> {
    if name.len() < 3
        || name.len() > 8
        || name.chars().any(|c| c < 'A' && c > 'Z')
    {
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
