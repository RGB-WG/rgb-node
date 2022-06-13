// RGB standard library
// Written in 2019-2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#![recursion_limit = "256"]
// Coding conventions
#![allow(dead_code, deprecated)]
#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    // dead_code
    // missing_docs,
)]

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;
#[cfg_attr(feature = "_rpc", macro_use)]
extern crate internet2;

#[cfg(feature = "clap")]
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

#[cfg(feature = "serde")]
extern crate serde_crate as serde;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_with;

#[cfg(all(feature = "fungibles", feature = "sql"))]
#[macro_use]
extern crate diesel;

#[cfg(feature = "cli")]
pub mod cli;
pub mod constants;
pub mod error;
#[cfg(feature = "node")]
pub mod i9n;
#[cfg(feature = "_rpc")]
pub mod rpc;
pub mod util;

#[cfg(all(feature = "_rpc", feature = "fungibles"))]
pub mod fungibled;
#[cfg(feature = "_rpc")]
pub mod rgbd;
#[cfg(feature = "node")]
pub mod stashd;
