// RGB Node: sovereign smart contracts backend
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
// Written in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2020-2024 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2025 RGB Consortium, Switzerland. All rights reserved.
// Copyright (C) 2020-2025 Dr Maxim Orlovsky. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

#[cfg(all(feature = "embedded", feature = "server"))]
compile_error!("Either `embedded` or `server` feature must be used");
#[cfg(not(any(feature = "embedded", feature = "server")))]
compile_error!("Either `embedded` or `server` feature must be used");

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate serde;

mod config;
pub mod services;
mod workers;
mod reactors;

pub use config::Config;
pub use reactors::*;
pub use workers::*;

pub type ReqId = u64;
