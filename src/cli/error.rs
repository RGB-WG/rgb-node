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

use crate::error::ServiceErrorDomain;

#[derive(Debug, Display, Error, From)]
#[display_from(Debug)]
pub enum Error {
    InputFileIoError(String),

    InputFileFormatError(String, String),

    #[derive_from]
    ServiceError(ServiceErrorDomain),

    #[derive_from]
    YamlError(serde_yaml::Error),

    #[derive_from]
    JsonError(serde_json::Error),

    #[derive_from(toml::de::Error, toml::ser::Error)]
    TomlError,

    #[derive_from]
    StrictEncoding(lnpbp::strict_encoding::Error),

    #[derive_from]
    ConsensusEncoding(lnpbp::bitcoin::consensus::encode::Error),

    DataInconsistency,

    UnsupportedFunctionality,
}
