// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#![recursion_limit = "256"]

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;
#[macro_use]
extern crate internet2;
#[macro_use]
extern crate log;

#[cfg(feature = "serde")]
extern crate serde_crate as serde;

pub mod client;
mod error;
mod messages;
mod service_id;

pub use client::Client;
pub use error::{Error, FailureCode};
pub(crate) use messages::BusMsg;
pub use messages::{
    AcceptReq, ComposeReq, ContractValidity, HelloReq, OutpointFilter, RpcMsg, TransferFinalize,
    TransferReq,
};
pub use service_id::ServiceId;

pub const RGB_NODE_RPC_ENDPOINT: &str = "0.0.0.0:63963";
