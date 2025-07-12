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

use std::io;
use std::io::{Read, Write};

use netservices::Frame;

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub(crate) struct CborFrame(pub Vec<u8>);

impl Frame for CborFrame {
    type Error = io::Error;

    fn unmarshall(mut reader: impl Read) -> Result<Option<Self>, Self::Error> {
        let mut buf = [0u8; 4];
        if reader
            .read_exact(&mut buf)
            .map(Some)
            .or_else(|e| if e.kind() == io::ErrorKind::UnexpectedEof { Ok(None) } else { Err(e) })?
            .is_none()
        {
            eprintln!("Incomplete frame header; skipping");
            return Ok(None);
        }
        let len = u32::from_be_bytes(buf) as usize;
        eprint!("Reading {len} bytes: ");
        let mut buf = vec![0u8; len];
        reader
            .read_exact(&mut buf)
            .map(|_| {
                eprintln!("{buf:02x?}");
                Some(CborFrame(buf))
            })
            .or_else(|e| if e.kind() == io::ErrorKind::UnexpectedEof { Ok(None) } else { Err(e) })
    }

    fn marshall(&self, mut writer: impl Write) -> Result<(), Self::Error> {
        let len = self.0.len() as u32;
        writer.write_all(&len.to_be_bytes())?;
        writer.write_all(&self.0)
    }
}
