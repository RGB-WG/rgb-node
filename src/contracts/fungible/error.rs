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

/*
pub use crate::error::{ApiErrorType, ServiceSocketType};

#[derive(Debug, Display, Error, From)]
#[display_from(Display)]
pub enum SchemaError {
    #[derive_from(core::option::NoneError)]
    NotAllFieldsPresent,
}

#[derive(Debug, Display, Error, From)]
#[display_from(Display)]
pub enum Error {
    Zmq(ServiceSocketType, String, zmq::Error),

    Api(ApiErrorType, String),

    #[derive_from]
    Secp(lnpbp::secp256k1zkp::Error),

    #[derive_from]
    SchemaError(SchemaError),
}

impl ServiceErrorDomain {
    pub fn zmq_request(socket: &str, err: zmq::Error) -> Self {
        Self::Zmq(ServiceSocketType::Request, socket.to_string(), err)
    }

    pub fn zmq_reply(socket: &str, err: zmq::Error) -> Self {
        Self::Zmq(ServiceSocketType::Request, socket.to_string(), err)
    }

    pub fn zmq_publish(socket: &str, err: zmq::Error) -> Self {
        Self::Zmq(ServiceSocketType::Request, socket.to_string(), err)
    }

    pub fn zmq_subscribe(socket: &str, err: zmq::Error) -> Self {
        Self::Zmq(ServiceSocketType::Request, socket.to_string(), err)
    }
}
*/
