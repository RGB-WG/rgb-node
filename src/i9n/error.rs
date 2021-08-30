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
use crate::rpc::reply;

#[derive(Debug, Display, Error, From)]
#[display(inner)]
pub enum Error {
    /// Integration module internal error
    #[from]
    ServiceError(ServiceErrorDomain),

    /// RGB Node returned error: {0}
    #[display(doc_comments)]
    #[from]
    Reply(reply::Failure),

    /// Error decoding the provided data from Base64 encoding
    #[from]
    Base64(base64::DecodeError),

    /// Error decoding the provided data from bitcoin consensus encoding
    #[from]
    Bitcoin(bitcoin::consensus::encode::Error),

    /// Error decoding the provided data from LNP/BP strict encoding
    #[from]
    Encoding(strict_encoding::Error),

    /// Unexpected server response; please check that RGB node uses the same
    /// API version as the client
    #[display(doc_comments)]
    UnexpectedResponse,

    /// The provided network id does not match the network used by the RGB node
    #[display(doc_comments)]
    WrongNetwork,
}
