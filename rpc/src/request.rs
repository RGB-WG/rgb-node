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

use std::io::{Read, Write};

use amplify::confinement::{SmallBlob, TinyBlob};
use bpstd::{DescrId, Network};
use netservices::Frame;
use rgbp::descriptors::RgbDescr;
use sonicapi::{CodexId, ContractId};

use crate::CiboriumError;

#[derive(Clone, Debug, Display)]
#[display(UPPERCASE)]
#[derive(Serialize, Deserialize)]
pub enum RgbRpcReq {
    #[display("HELLO({0})")]
    Hello(Network),

    #[display("PING")]
    Ping(TinyBlob),

    Status,

    Wallets,

    #[display("WALLET({0})")]
    Wallet(DescrId),

    #[display("CREATE({0})")]
    Create(RgbDescr),

    #[display("DELETE({0})")]
    Delete(DescrId),

    Issuers,

    Contracts,

    #[display("ISSUER({0})")]
    Issuer(CodexId),

    #[display("ARTICLES({0})")]
    Articles(ContractId),

    #[display("CONTRACT({0})")]
    Contract(ContractId),

    #[display("CONSIGN({0}, ...)")]
    Consign(u64, ContractId),

    /*
    #[display("IMPORT(...)")]
    Import(Issuer),
     */
    #[display("ACCEPT_INIT({0})")]
    AcceptInit(ContractId),

    #[display("ACCEPT_DATA")]
    AcceptData(u64, SmallBlob),
}

impl Frame for RgbRpcReq {
    type Error = CiboriumError;

    fn unmarshall(reader: impl Read) -> Result<Option<Self>, Self::Error> {
        ciborium::from_reader(reader).map_err(CiboriumError::from)
    }

    fn marshall(&self, writer: impl Write) -> Result<(), Self::Error> {
        ciborium::into_writer(self, writer)?;
        Ok(())
    }
}
