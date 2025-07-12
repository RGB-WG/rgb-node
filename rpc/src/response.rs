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

use std::collections::BTreeMap;
use std::io::{ErrorKind, Read, Write};

use amplify::confinement::{SmallBlob, TinyBlob};
use bpstd::DescrId;
use bpstd::psbt::Utxo;
use bpstd::seals::TxoSeal;
use netservices::Frame;
use rgb::{ContractState, ContractStateName, WitnessStatus};
use rgbp::ContractInfo;
use rgbp::descriptors::RgbDescr;
use sonicapi::{CellAddr, CodexId, ContractId, Opid, StateAtom};
use strict_types::StrictVal;

use crate::{CiboriumError, Failure, Status};

#[derive(Clone, Debug, Display)]
#[derive(Serialize, Deserialize)]
pub enum RgbRpcResp {
    #[display("MESSAGE({0})")]
    Message(String),

    #[display("FAILURE({0})")]
    Failure(Failure),

    #[display("PONG")]
    Pong(TinyBlob),

    #[display("STATUS")]
    Status(Status),

    #[display("WALLETS(...)")]
    Wallets(Vec<WalletInfo>),

    #[display("WALLET_STATE({0}, ...)")]
    WalletState(DescrId, WalletState),

    #[display("ISSUERS(...)")]
    Issuers(Vec<CodexId>),

    /*
    #[display("ISSUER(...)")]
    Issuer(Schema),

    #[display("ARTICLES(...)")]
    Articles(Articles),
     */
    #[display("CONTRACTS(...)")]
    Contracts(Vec<ContractInfo>),

    #[display("CONTRACT_STATE({0}, ...)")]
    ContractState(ContractId, ContractState<TxoSeal>),

    #[display("CONSIGN_INIT({0})")]
    ConsignInit(u64),

    #[display("CONSIGN_DATA({0}, ...)")]
    ConsignData(u64, SmallBlob),

    #[display("IMPORTED({0})")]
    Imported(CodexId),

    #[display("ACCEPT_START({0})")]
    AcceptStart(u64),

    #[display("ACCEPT_COMPLETE({0})")]
    AcceptComplete(u64),
}

impl Frame for RgbRpcResp {
    type Error = CiboriumError;

    fn unmarshall(reader: impl Read) -> Result<Option<Self>, Self::Error> {
        match ciborium::from_reader(reader) {
            Ok(msg) => Ok(Some(msg)),
            Err(ciborium::de::Error::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(CiboriumError::from(e)),
        }
    }

    fn marshall(&self, writer: impl Write) -> Result<(), Self::Error> {
        ciborium::into_writer(self, writer)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: DescrId,
    pub name: String,
    pub descriptor: RgbDescr,
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct WalletState {
    pub info: WalletInfo,
    pub immutable: BTreeMap<ContractStateName, BTreeMap<CellAddr, StateAtom>>,
    pub owned: BTreeMap<Utxo, BTreeMap<ContractStateName, BTreeMap<CellAddr, StrictVal>>>,
    pub aggregated: BTreeMap<ContractStateName, StrictVal>,
    pub confirmations: BTreeMap<Opid, WitnessStatus>,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn serialization() {
        let mut buf = Vec::new();
        RgbRpcResp::Message(s!("Test")).marshall(&mut buf).unwrap();
        assert_eq!(buf, *b"\xA1\x67Message\x64Test");
        let deser = RgbRpcResp::unmarshall(&mut buf.as_slice())
            .unwrap()
            .unwrap();
        assert!(matches!(deser, RgbRpcResp::Message(m) if m == "Test"));
    }

    #[test]
    fn stream_serialization() {
        let mut buf = Vec::new();
        RgbRpcResp::Message(s!("Test")).marshall(&mut buf).unwrap();
        assert_eq!(buf, *b"\xA1\x67Message\x64Test");
        let mut cursor = Cursor::new(&mut buf);
        let deser = RgbRpcResp::unmarshall(&mut cursor).unwrap().unwrap();
        assert!(matches!(deser, RgbRpcResp::Message(m) if m == "Test"));
        let nothing = RgbRpcResp::unmarshall(&mut cursor).unwrap();
        assert!(matches!(nothing, None));
    }
}
