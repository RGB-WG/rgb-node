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
pub struct Outcoins {
    pub coins: f32,
    pub vout: u16,
    pub txid: Option<Txid>,
}

impl Outcoins {
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

impl FromStr for Outcoins {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(
            r"(?x)
                ^(?P<coins>[\d.,_']+) # float amount
                @
                ((?P<txid>[a-f\d]{64}) # Txid
                :)
                (?P<vout>\d+)$ # Vout
            ",
        )
        .expect("Regex parse failure");
        if let Some(m) = re.captures(&s.to_ascii_lowercase()) {
            match (m.name("coins"), m.name("txid"), m.name("vout")) {
                (Some(amount), Some(txid), Some(vout)) => Ok(Self {
                    coins: amount.as_str().parse()?,
                    vout: vout.as_str().parse()?,
                    txid: Some(Txid::from_hex(txid.as_str())?),
                }),
                (Some(amount), None, Some(vout)) => Ok(Self {
                    coins: amount.as_str().parse()?,
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
