// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;
#[macro_use]
extern crate internet2;
#[macro_use]
extern crate log;

mod config;
mod error;
pub mod rgbd;
pub mod bus;
pub mod bucketd;
#[cfg(feature = "server")]
pub mod opts;
pub(crate) mod db;

pub use config::Config;
pub(crate) use error::DaemonError;
pub use error::LaunchError;
