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
use rgb_rpc::{Client, FailureCode};

use crate::{Command, Opts};

impl Exec for Opts {
    type Client = Client;
    type Error = ServerError<FailureCode>;

    fn exec(self, _runtime: &mut Self::Client) -> Result<(), Self::Error> {
        debug!("Performing {:?}", self.command);
        match self.command {
            Command::None => {}
        }
        Ok(())
    }
}
