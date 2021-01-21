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
#[display(Debug)]
pub enum Error {
    InputFileIoError(String),

    InputFileFormatError(String, String),

    #[from]
    ServiceError(ServiceErrorDomain),

    #[from]
    YamlError(serde_yaml::Error),

    #[from]
    JsonError(serde_json::Error),

    #[from(toml::de::Error)]
    #[from(toml::ser::Error)]
    TomlError,

    #[from]
    StrictEncoding(lnpbp::strict_encoding::Error),

    #[from]
    ConsensusEncoding(bitcoin::consensus::encode::Error),

    DataInconsistency,

    UnsupportedFunctionality,

    FormatNotSupported,
}
