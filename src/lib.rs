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

#![allow(dead_code)]

#[cfg(feature = "clap")]
#[macro_use]
extern crate clap;
#[macro_use]
extern crate amplify;
#[macro_use]
extern crate amplify_derive;
#[macro_use]
extern crate lnpbp;
#[macro_use]
extern crate internet2;
#[cfg(feature = "async-trait")]
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate log;
#[cfg(feature = "serde")]
extern crate serde_crate as serde;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_with;

#[macro_use]
pub extern crate diesel;

extern crate hammersbald;

#[cfg(feature = "cli")]
pub mod cli;
pub mod constants;
pub mod error;
#[cfg(any(feature = "node"))]
pub mod i9n;
#[cfg(any(feature = "node", feature = "client"))]
pub mod rpc;
pub mod util;

#[cfg(all(any(feature = "node", feature = "client"), feature = "fungibles"))]
pub mod fungibled;
#[cfg(any(feature = "node", feature = "client"))]
pub mod rgbd;
#[cfg(any(feature = "node", feature = "client"))]
pub mod service;
#[cfg(feature = "node")]
pub mod stashd;

use std::str::FromStr;

#[derive(
    Clap,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[non_exhaustive]
pub enum DataFormat {
    /// JSON
    #[cfg(feature = "serde_json")]
    #[display("json")]
    Json,

    /// YAML
    #[cfg(feature = "serde_yaml")]
    #[display("yaml")]
    Yaml,

    /// TOML
    #[cfg(feature = "toml")]
    #[display("toml")]
    Toml,

    /// Strict encoding
    #[display("strict-encode")]
    StrictEncode,
}

impl DataFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            #[cfg(feature = "serde_yaml")]
            DataFormat::Yaml => "yaml",
            #[cfg(feature = "serde_json")]
            DataFormat::Json => "json",
            #[cfg(feature = "toml")]
            DataFormat::Toml => "toml",
            DataFormat::StrictEncode => "se",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum FileFormatParseError {
    /// Unknown file format
    UnknownFormat,
}

impl FromStr for DataFormat {
    type Err = FileFormatParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match &s.to_lowercase() {
            #[cfg(feature = "serde_yaml")]
            s if s.starts_with("yaml") || s.starts_with("yml") => Self::Yaml,
            #[cfg(feature = "serde_json")]
            s if s.starts_with("json") => Self::Json,
            #[cfg(feature = "toml")]
            s if s.starts_with("toml") => Self::Toml,
            s if s.starts_with("se")
                || s.starts_with("dat")
                || s.starts_with("strictencode")
                || s.starts_with("strict-encode")
                || s.starts_with("strict_encode") =>
            {
                Self::StrictEncode
            }
            _ => Err(FileFormatParseError::UnknownFormat)?,
        })
    }
}
