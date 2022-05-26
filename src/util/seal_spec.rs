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

// TODO #166: Consider moving this to LNP/BP Core Library

use core::str::FromStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use bitcoin::{OutPoint, Txid};
use rgb::contract::seal::Revealed;

#[derive(Clone, Copy, Debug, Display, Error, From)]
#[display(doc_comments)]
#[from(std::num::ParseFloatError)]
#[from(std::num::ParseIntError)]
#[from(bitcoin::blockdata::transaction::ParseOutPointError)]
#[from(bitcoin::hashes::hex::Error)]
/// Error parsing seal specification; it must be either a integer (output
/// number) or transaction outpoint in form of `txid:vout`, where `txid` must be
/// a hexadecimal string.
pub struct ParseError;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Display, StrictEncode, StrictDecode,
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize,),
    serde(crate = "serde_crate")
)]
#[display(Debug)]
pub struct SealSpec {
    pub vout: u32,
    pub txid: Option<Txid>,
}

impl SealSpec {
    pub fn with_vout(vout: u32) -> Self {
        Self { vout, txid: None }
    }
}

impl From<OutPoint> for SealSpec {
    fn from(outpoint: OutPoint) -> Self {
        Self {
            vout: outpoint.vout,
            txid: Some(outpoint.txid),
        }
    }
}

impl From<OutpointReveal> for SealSpec {
    fn from(revealed: OutpointReveal) -> Self {
        Self {
            vout: revealed.vout,
            txid: Some(revealed.txid),
        }
    }
}

impl From<SealDefinition> for SealSpec {
    fn from(seal: SealDefinition) -> Self {
        match seal {
            Revealed::TxOutpoint(revealed) => revealed.into(),
            Revealed::WitnessVout { vout, .. } => SealSpec::with_vout(vout),
        }
    }
}

impl SealSpec {
    pub fn seal_definition(&self) -> SealDefinition {
        use bitcoin::secp256k1::rand::{self, RngCore};
        let mut rng = rand::thread_rng();
        let entropy = rng.next_u64(); // Not an amount blinding factor but outpoint blinding
        match self.txid {
            Some(txid) => SealDefinition::TxOutpoint(OutpointReveal {
                blinding: entropy,
                txid,
                vout: self.vout,
            }),
            None => SealDefinition::WitnessVout {
                vout: self.vout,
                blinding: entropy,
            },
        }
    }
}

impl FromStr for SealSpec {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(outpoint) = OutPoint::from_str(s) {
            Ok(outpoint.into())
        } else {
            Ok(SealSpec::with_vout(s.parse()?))
        }
    }
}
