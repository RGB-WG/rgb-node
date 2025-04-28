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

#[cfg(not(feature = "embedded"))]
use std::net::SocketAddr;
use std::path::PathBuf;

use bpwallet::Network;

/// Final configuration resulting from data contained in config file environment variables and
/// command-line options.
/// For security reasons a node key is kept separately.
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    /// Data location
    pub data_dir: PathBuf,

    pub network: Network,

    #[cfg(not(feature = "embedded"))]
    pub rpc: Vec<SocketAddr>,
}

impl Config {
    pub fn data_dir(&self) -> PathBuf {
        let data_dir = self.data_dir.to_str().expect("Invalid data dir");
        #[cfg(not(feature = "embedded"))]
        let data_dir = shellexpand::full(&data_dir).expect("Invalid data dir");
        PathBuf::from(data_dir.to_string()).join(self.network.to_string())
    }
}
