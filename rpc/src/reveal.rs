// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use bitcoin::OutPoint;
use bp::seals::txout::CloseMethod;

#[derive(From, PartialEq, Eq, Debug, Clone, StrictEncode, StrictDecode)]
pub struct Reveal {
    /// Outpoint blinding factor (generated when the utxo blinded was created)
    pub blinding_factor: u64,

    /// Locally-controlled outpoint (specified when the utxo blinded was created)
    pub outpoint: OutPoint,

    /// method (specified when the utxo blinded was created)
    pub close_method: CloseMethod,
}

impl std::fmt::Display for Reveal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}#{}", self.close_method, self.outpoint, self.blinding_factor)
    }
}

/// Parses a blinding factor.
fn parse_blind(s: &str) -> Result<u64, ParseRevealError> {
    s.parse().map_err(ParseRevealError::BlindingFactor)
}

impl ::core::str::FromStr for Reveal {
    type Err = ParseRevealError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 9 + 19 + 1 + 64 + 1 + 10
        if s.len() > 97 {
            return Err(ParseRevealError::TooLong);
        }
        let find_method = s.find('@');
        if find_method == None {
            return Err(ParseRevealError::Format);
        }

        let colon_method = find_method.unwrap();
        if colon_method == 0 || colon_method == s.len() - 1 {
            return Err(ParseRevealError::Format);
        }

        let find_blind = s.find('#');
        if find_blind == None {
            return Err(ParseRevealError::Format);
        }

        let colon_blind = find_blind.unwrap();
        if colon_blind == 0 || colon_blind == s.len() - 1 {
            return Err(ParseRevealError::Format);
        }

        Ok(Reveal {
            close_method: match CloseMethod::from_str(&s[..colon_method]) {
                Ok(it) => it,
                Err(_) => return Err(ParseRevealError::CloseMethod),
            },
            outpoint: match OutPoint::from_str(&s[colon_method + 1..colon_blind]) {
                Ok(it) => it,
                Err(_) => return Err(ParseRevealError::Outpoint),
            },
            blinding_factor: parse_blind(&s[colon_blind + 1..])?,
        })
    }
}

/// An error in parsing an OutPoint.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParseRevealError {
    /// Error in outpoint part.
    CloseMethod,
    /// Error in outpoint part.
    Outpoint,
    /// Error in blinding factor part.
    BlindingFactor(::core::num::ParseIntError),
    /// Error in general format.
    Format,
    /// Size exceeds max.
    TooLong,
}

impl std::fmt::Display for ParseRevealError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ParseRevealError::CloseMethod => write!(f, "error parsing CloseMethod"),
            ParseRevealError::Outpoint => write!(f, "error parsing OutPoint"),
            ParseRevealError::BlindingFactor(ref e) => {
                write!(f, "error parsing blinding_factor: {}", e)
            }
            ParseRevealError::Format => {
                write!(f, "Reveal not in <blind_factor>@<txid>:<vout> format")
            }
            ParseRevealError::TooLong => write!(f, "reveal should be at most 95 digits"),
        }
    }
}

impl ::std::error::Error for ParseRevealError {
    fn cause(&self) -> Option<&dyn ::std::error::Error> {
        match *self {
            ParseRevealError::BlindingFactor(ref e) => Some(e),
            _ => None,
        }
    }
}
