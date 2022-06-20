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
//#[cfg(feature = "serde")]
//#[macro_use]
//extern crate serde_with;

pub mod client;
mod error;
mod messages;
mod service_id;

pub use client::Client;
pub use error::{Error, FailureCode};
pub(crate) use messages::BusMsg;
pub use messages::{OptionDetails, RpcMsg};
pub use service_id::{ClientId, ServiceId, ServiceName};

#[cfg(any(target_os = "linux"))]
pub const RGB_NODE_DATA_DIR: &str = "~/.rgb";
#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub const RGB_NODE_DATA_DIR: &str = "~/.rgb";
#[cfg(target_os = "macos")]
pub const RGB_NODE_DATA_DIR: &str = "~/Library/Application Support/RGB Node";
#[cfg(target_os = "windows")]
pub const RGB_NODE_DATA_DIR: &str = "~\\AppData\\Local\\RGB Node";
#[cfg(target_os = "ios")]
pub const RGB_NODE_DATA_DIR: &str = "~/Documents";
#[cfg(target_os = "android")]
pub const RGB_NODE_DATA_DIR: &str = ".";

// TODO: Change port
pub const RGB_NODE_RPC_ENDPOINT: &str = const_format::concatcp!(RGB_NODE_DATA_DIR, "/rpc");
