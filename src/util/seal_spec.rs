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
use serde::{Deserialize, Serialize};
use std::io;

use bitcoin::hashes::hex::FromHex;
use bitcoin::Txid;
use lnpbp::bitcoin;
use lnpbp::bp;
use lnpbp::rgb::SealDefinition;
use lnpbp::strict_encoding::{self, StrictDecode, StrictEncode};

use crate::error::ParseError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Display)]
#[display_from(Debug)]
pub struct SealSpec {
    pub vout: u16,
    pub txid: Option<Txid>,
}

impl SealSpec {
    pub fn seal_definition(&self) -> SealDefinition {
        use lnpbp::bitcoin::secp256k1::rand::{self, RngCore};
        let mut rng = rand::thread_rng();
        let entropy = rng.next_u32(); // Not an amount blinding factor but outpoint blinding
        match self.txid {
            Some(txid) => SealDefinition::TxOutpoint(bp::blind::OutpointReveal {
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

impl StrictEncode for SealSpec {
    type Error = strict_encoding::Error;

    fn strict_encode<E: io::Write>(&self, mut e: E) -> Result<usize, Self::Error> {
        Ok(strict_encode_list!(e; self.vout, self.txid))
    }
}

impl StrictDecode for SealSpec {
    type Error = strict_encoding::Error;

    fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Self::Error> {
        Ok(Self {
            vout: u16::strict_decode(&mut d)?,
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
