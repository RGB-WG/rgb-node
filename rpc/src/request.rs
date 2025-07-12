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

use std::io::{Cursor, ErrorKind, Read, Write};

use amplify::confinement::{SmallBlob, TinyBlob};
use bpstd::DescrId;
use netservices::Frame;
use rgbp::descriptors::RgbDescr;
use sonicapi::{CodexId, ContractId};

use crate::AgentInfo;
use crate::frame::CborFrame;

#[derive(Clone, Debug, Display)]
#[derive(Serialize, Deserialize)]
pub enum RgbRpcReq {
    #[display("HELLO({0})")]
    Hello(AgentInfo),

    #[display("PING")]
    Ping(TinyBlob),

    #[display("STATUS")]
    Status,

    #[display("WALLETS")]
    Wallets,

    #[display("WALLET({0})")]
    Wallet(DescrId),

    #[display("CREATE({0})")]
    Create(RgbDescr),

    #[display("DELETE({0})")]
    Delete(DescrId),

    #[display("ISSUERS")]
    Issuers,

    #[display("CONTRACTS")]
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
    type Error = serde_cbor_2::Error;

    fn unmarshall(reader: impl Read) -> Result<Option<Self>, Self::Error> {
        let Some(frame) = CborFrame::unmarshall(reader)? else {
            return Ok(None);
        };
        let cursor = Cursor::new(frame.0);
        serde_cbor_2::from_reader(cursor).map(Some)
    }

    fn marshall(&self, writer: impl Write) -> Result<(), Self::Error> {
        let mut buf = Vec::with_capacity(4096);
        serde_cbor_2::to_writer(&mut buf, self)?;
        CborFrame(buf)
            .marshall(writer)
            .map_err(serde_cbor_2::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn partial_serialization() {
        let cursor = Cursor::new(*b"\0\0\0\x08\x67Wallet");
        let deser = RgbRpcReq::unmarshall(cursor).unwrap();
        assert!(matches!(deser, None));
    }

    #[test]
    fn single_serialization() {
        let mut buf = Vec::new();
        RgbRpcReq::Wallets.marshall(&mut buf).unwrap();
        assert_eq!(buf, *b"\0\0\0\x08\x67Wallets");
        let deser = RgbRpcReq::unmarshall(&mut buf.as_slice()).unwrap().unwrap();
        assert!(matches!(deser, RgbRpcReq::Wallets));
    }

    #[test]
    fn stream_serialization() {
        let mut buf = Vec::new();
        RgbRpcReq::Wallets.marshall(&mut buf).unwrap();
        assert_eq!(buf, *b"\0\0\0\x08\x67Wallets");
        let mut cursor = Cursor::new(&mut buf);
        let deser = RgbRpcReq::unmarshall(&mut cursor).unwrap().unwrap();
        assert!(matches!(deser, RgbRpcReq::Wallets));
        let nothing = RgbRpcReq::unmarshall(&mut cursor).unwrap();
        assert!(matches!(nothing, None));
    }

    #[test]
    fn multi_serialization() {
        let mut buf = Vec::new();
        RgbRpcReq::Wallets.marshall(&mut buf).unwrap();
        RgbRpcReq::Wallets.marshall(&mut buf).unwrap();
        assert_eq!(buf, *b"\0\0\0\x08\x67Wallets\0\0\0\x08\x67Wallets");
        let mut cursor = Cursor::new(&mut buf);
        let deser = RgbRpcReq::unmarshall(&mut cursor).unwrap().unwrap();
        assert!(matches!(deser, RgbRpcReq::Wallets));
        let deser2 = RgbRpcReq::unmarshall(&mut cursor).unwrap().unwrap();
        assert!(matches!(deser2, RgbRpcReq::Wallets));
    }
}
