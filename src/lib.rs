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

#![feature(never_type)]
#![feature(unwrap_infallible)]
#![feature(in_band_lifetimes)]
#![feature(try_trait)]
#![feature(with_options)]

extern crate futures;
extern crate zmq;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate derive_wrapper;
extern crate chrono;
extern crate lightning_invoice;
extern crate num_derive;
extern crate num_traits;
extern crate regex;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate dotenv;
extern crate env_logger;

#[macro_use]
extern crate lnpbp;
extern crate bech32;

mod api;
pub mod cli;
mod contracts;
mod error;
pub mod i9n;
pub mod stashd;
pub mod util;

pub use contracts::*;
pub use error::BootstrapError;

pub const RGB_BECH32_HRP_GENESIS: &'static str = "rgb:genesis";
