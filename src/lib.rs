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

#![feature(
    never_type,
    unwrap_infallible,
    in_band_lifetimes,
    try_trait,
    with_options
)]

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate derive_wrapper;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate async_trait;
#[macro_use]
#[macro_use]
extern crate log;

#[macro_use]
pub extern crate lnpbp;

mod api;
pub mod cli;
pub mod constants;
mod contracts;
pub mod error;
pub mod i9n;
pub mod rgbd;
pub mod stash;
pub mod util;

pub use contracts::*;
