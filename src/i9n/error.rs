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

use crate::api::reply;
use crate::error::ServiceErrorDomain;

#[derive(Debug, Display, Error, From)]
#[display_from(Debug)]
pub enum Error {
    #[derive_from]
    ServiceError(ServiceErrorDomain),

    #[derive_from]
    Reply(reply::Failure),

    #[derive_from]
    Base64(base64::DecodeError),

    #[derive_from]
    Bitcoin(lnpbp::bitcoin::consensus::encode::Error),

    #[derive_from]
    Encoding(lnpbp::strict_encoding::Error),

    UnexpectedResponse,
}
