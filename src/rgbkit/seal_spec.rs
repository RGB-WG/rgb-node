use core::str::FromStr;
use regex::Regex;
use serde::{Deserialize, Serialize};

use bitcoin::hashes::hex::FromHex;
use bitcoin::Txid;
use lnpbp::bitcoin;
use lnpbp::bp;
use lnpbp::rgb::SealDefinition;

use crate::rgbkit::ParseError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Display)]
#[display_from(Debug)]
pub struct SealSpec {
    pub vout: u16,
    pub txid: Option<Txid>,
}

impl SealSpec {
    pub fn seal_definition(&self) -> SealDefinition {
        use lnpbp::rand::{self, RngCore};
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
