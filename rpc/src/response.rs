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

use amplify::confinement::{
    MediumOrdSet, MediumVec, SmallBlob, SmallOrdSet, TinyBlob, U24 as U24MAX,
};
use netservices::Frame;
use sonicapi::{CodexId, ContractId};
use strict_encoding::{
    DecodeError, StreamReader, StreamWriter, StrictDecode, StrictEncode, StrictReader, StrictWriter,
};

use crate::{Failure, RGB_RPC_LIB, Status};

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = RGB_RPC_LIB, tags = custom, dumb = Self::Pong(strict_dumb!()))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RgbRpcResp {
    #[display("FAILURE({0})")]
    #[strict_type(tag = 0x00)]
    Failure(Failure),

    #[display("PONG")]
    #[strict_type(tag = 0x01)]
    Pong(TinyBlob),

    #[display("STATUS")]
    #[strict_type(tag = 0x03)]
    Status(Status),

    #[display("ISSUERS(...)")]
    #[strict_type(tag = 0x10)]
    Issuers(SmallOrdSet<CodexId>),

    #[display("CONTRACTS(...)")]
    #[strict_type(tag = 0x12)]
    Contracts(MediumOrdSet<ContractId>),

    /*
    #[strict_type(tag = 0x20)]
    #[display("ISSUER(...)")]
    Issuer(Schema),

    #[strict_type(tag = 0x22)]
    #[display("ARTICLES(...)")]
    Articles(Articles),
     */
    #[display("STATE({0})")]
    #[strict_type(tag = 0x24)]
    State(ContractReply),

    #[display("CONSIGN_INIT({0})")]
    #[strict_type(tag = 0x30)]
    ConsignInit(u64),

    #[display("CONSIGN_DATA({0}, ...)")]
    #[strict_type(tag = 0x32)]
    ConsignData(u64, SmallBlob),

    #[display("IMPORTED({0})")]
    #[strict_type(tag = 0x40)]
    Imported(CodexId),

    #[display("ACCEPT_START({0})")]
    #[strict_type(tag = 0x42)]
    AcceptStart(u64),

    #[display("ACCEPT_COMPLETE({0})")]
    #[strict_type(tag = 0x44)]
    AcceptComplete(u64),
}

impl Frame for RgbRpcResp {
    type Error = DecodeError;

    fn unmarshall(reader: impl Read) -> Result<Option<Self>, Self::Error> {
        let mut reader = StrictReader::with(StreamReader::new::<U24MAX>(reader));
        match Self::strict_decode(&mut reader) {
            Ok(request) => Ok(Some(request)),
            Err(DecodeError::Io(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn marshall(&self, writer: impl Write) -> Result<(), Self::Error> {
        let writer = StrictWriter::with(StreamWriter::new::<U24MAX>(writer));
        self.strict_encode(writer)?;
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display("{contract_id}, ...")]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = RGB_RPC_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ContractReply {
    pub contract_id: ContractId,
    /// Reply data in bincode format
    pub data: MediumVec<u8>,
}
