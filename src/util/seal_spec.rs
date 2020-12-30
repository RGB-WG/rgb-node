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

use core::str::FromStr;
use regex::Regex;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::io;

use lnpbp::bitcoin::Txid;
use lnpbp::bp;
use lnpbp::hex::FromHex;
use lnpbp::rgb::SealDefinition;
use lnpbp::strict_encoding::{self, StrictDecode, StrictEncode};

use crate::error::ParseError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
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
    pub fn seal_definition(&self) -> SealDefinition {
        use lnpbp::bitcoin::secp256k1::rand::{self, RngCore};
        let mut rng = rand::thread_rng();
        let entropy = rng.next_u64(); // Not an amount blinding factor but outpoint blinding
        match self.txid {
            Some(txid) => {
                SealDefinition::TxOutpoint(bp::blind::OutpointReveal {
                    blinding: entropy,
                    txid,
                    vout: self.vout,
                })
            }
            None => SealDefinition::WitnessVout {
                vout: self.vout,
                blinding: entropy,
            },
        }
    }
}

impl StrictEncode for SealSpec {
    fn strict_encode<E: io::Write>(
        &self,
        mut e: E,
    ) -> Result<usize, strict_encoding::Error> {
        Ok(strict_encode_list!(e; self.vout, self.txid))
    }
}

impl StrictDecode for SealSpec {
    fn strict_decode<D: io::Read>(
        mut d: D,
    ) -> Result<Self, strict_encoding::Error> {
        Ok(Self {
            vout: u32::strict_decode(&mut d)?,
            txid: Option::<Txid>::strict_decode(&mut d)?,
        })
    }
}

impl FromStr for SealSpec {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(
            r"(?x)
                ((?P<txid>[a-f\d]{64}) # Txid
                :)
                (?P<vout>\d+)$ # Vout
            ",
        )
        .expect("Regex parse failure");
        if let Some(m) = re.captures(&s.to_ascii_lowercase()) {
            match (m.name("txid"), m.name("vout")) {
                (Some(txid), Some(vout)) => Ok(Self {
                    vout: vout.as_str().parse()?,
                    txid: Some(Txid::from_hex(txid.as_str())?),
                }),
                (None, Some(vout)) => Ok(Self {
                    vout: vout.as_str().parse()?,
                    txid: None,
                }),
                _ => Err(ParseError),
            }
        } else {
            Err(ParseError)
        }
    }
}
