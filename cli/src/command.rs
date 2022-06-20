// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use microservices::rpc::ServerError;
use microservices::shell::Exec;
use rgb_rpc::{Client, FailureCode, RpcMsg};

use crate::{Command, Opts};

impl Command {
    pub fn action_string(&self) -> String {
        match self {
            Command::Register { contract } => {
                format!("Registering contract {}", contract.contract_id())
            }
        }
    }
}

impl Exec for Opts {
    type Client = Client;
    type Error = ServerError<FailureCode>;

    fn exec(self, runtime: &mut Self::Client) -> Result<(), Self::Error> {
        eprintln!("{} ... ", self.command.action_string());
        let reply = match self.command {
            Command::Register { contract } => runtime.request(RpcMsg::AddContract(contract))?,
        };
        match reply {
            RpcMsg::ContractIds(_) => {}
            RpcMsg::Contract(_) => {}
            RpcMsg::ContractState(_) => {}
            RpcMsg::StateTransfer(_) => {}
            RpcMsg::Success => eprintln!("success"),
            RpcMsg::Failure(_) => {}
            wrong => unreachable!("unrecognized node reply {}", wrong),
        }
        Ok(())
    }
}
