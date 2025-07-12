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

use std::ops::ControlFlow;

use async_channel::{SendError as AsyncSendError, Sender as AsyncSender};
use crossbeam_channel::{Receiver, RecvError, SendError, Sender};
use microservices::{USender, UService};
use rgb::ContractId;
use rgbp::ContractInfo;
use rgbrpc::{Failure, RgbRpcReq, RgbRpcResp};

use crate::ReqId;

type Request = (AsyncSender<RgbRpcResp>, RgbRpcReq);

#[derive(Clone)]
pub struct AsyncClient {
    sender: USender<Request>,
}

impl AsyncClient {
    pub(crate) fn new(sender: USender<Request>) -> Self { Self { sender } }

    async fn request(&self, request: RgbRpcReq) -> RgbRpcResp {
        let (rx, tx) = async_channel::bounded(1);
        if let Err(err) = self.sender.send((rx, request)) {
            log::error!(target: "rgb-node", "Client channel is dead: {err}");
            return RgbRpcResp::Failure(Failure::internal_error("Client channel is dead"));
        }
        tx.recv().await.unwrap_or_else(|err| {
            log::error!(target: "rgb-node", "Client channel is dead: {err}");
            RgbRpcResp::Failure(Failure::internal_error("Client channel is dead"))
        })
    }

    pub async fn contract_info(&self) -> Vec<ContractInfo> {
        let RgbRpcResp::Contracts(ids) = self.request(RgbRpcReq::Contracts).await else {
            panic!("Unexpected response from RGB RPC server");
        };
        ids
    }
}

pub struct AsyncDispatcher {
    to_broker: Sender<(ReqId, RgbRpcReq)>,
    from_broker: Receiver<(ReqId, RgbRpcResp)>,
    last_req_id: ReqId,
}

impl UService for AsyncDispatcher {
    type Msg = Request;
    type Error = AsyncNodeError;
    const NAME: &'static str = "async-dispatcher";

    /// Processing requests from [`AsyncClient`].
    fn process(&mut self, (channel, request): Self::Msg) -> Result<ControlFlow<u8>, Self::Error> {
        let req_id = self.last_req_id;
        self.last_req_id += 1;
        if let Err(err) = self.to_broker.send((req_id, request)) {
            log::error!(target: Self::NAME, "Broker thread channel is dead: {err}");
            if let Err(err) = channel.send_blocking(RgbRpcResp::Failure(Failure::internal_error(
                "Broker thread channel is dead",
            ))) {
                log::error!(target: Self::NAME, "The responder channel is also dead: {err}");
            }
            return Err(err.into());
        }
        // Here we rely on the following assumption: since in `embedded` mode there is just a single
        // client, processing of requests synchronously, this will lead to the 1-to-1
        // mapping between requests and responses, and we can wait for the broker response before
        // returning here.
        match self.from_broker.recv() {
            Err(err) => {
                if let Err(err) = channel.send_blocking(RgbRpcResp::Failure(
                    Failure::internal_error("Broker thread channel is dead"),
                )) {
                    log::error!(target: Self::NAME, "The responder channel is also dead: {err}");
                }
                Err(err.into())
            }
            Ok((id, resp)) => {
                debug_assert_eq!(
                    id, req_id,
                    "Broker thread does not process requests sequentially"
                );
                if let Err(err) = channel.send_blocking(resp) {
                    return Err(err.into());
                }
                Ok(ControlFlow::Continue(()))
            }
        }
    }

    fn terminate(&mut self) {
        // TODO: Send client failures on all pending requests
    }
}

impl AsyncDispatcher {
    pub fn new(
        to_broker: Sender<(ReqId, RgbRpcReq)>,
        from_broker: Receiver<(ReqId, RgbRpcResp)>,
    ) -> Self {
        Self { last_req_id: 0, to_broker, from_broker }
    }
}

#[derive(Clone, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum AsyncNodeError {
    #[from]
    BrokerSend(SendError<(ReqId, RgbRpcReq)>),
    #[from]
    BrokerRecv(RecvError),
    #[from]
    ClientSend(AsyncSendError<RgbRpcResp>),
}
