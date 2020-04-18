// Kaleidoscope: RGB command-line wallet utility
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Alekos Filini <alekos.filini@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.


use std::fs::File;
use std::io::{self, prelude::*};
use std::convert::TryFrom;

use lnpbp::service::*;
use bitcoin::util::bip32::{ExtendedPubKey, ChildNumber};
use bitcoin::network::constants::Network;
use bitcoin::Address;
use bitcoin_wallet::{account::*, context::*};

use super::*;
use crate::constants::*;
use crate::error::BootstrapError;


#[derive(Debug, Clone, Eq, PartialEq)]
struct Error(String);
impl Error {
    fn from(str: &str) -> Self {
        return Error(String::from(str))
    }
}
impl<E: ToString> From<E> for Error {
    fn from(err: E) -> Self {
        Error(err.to_string())
    }
}
impl Into<io::Error> for Error {
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self.0)
    }
}

pub struct Runtime {
    config: Config,
    context: zmq::Context,
    api_socket: zmq::Socket,
    sub_socket: zmq::Socket,
}

impl Runtime {
    pub async fn init(config: Config) -> Result<Self, BootstrapError> {
        let context = zmq::Context::new();

        debug!("Opening API socket to bpd on {} ...", config.bpd_api);
        let api_socket = context.socket(zmq::REQ)
            .map_err(|e| BootstrapError::PublishingError(e))?;
        api_socket.bind(&config.bpd_api)
            .map_err(|e| BootstrapError::PublishingError(e))?;

        debug!("Opening push notification socket to bpd on {} ...", config.bpd_subscr);
        let sub_socket = context.socket(zmq::SUB)
            .map_err(|e| BootstrapError::SubscriptionError(e))?;
        sub_socket.connect(&config.bpd_subscr)
            .map_err(|e| BootstrapError::SubscriptionError(e))?;
        sub_socket.set_subscribe("".as_bytes())
            .map_err(|e| BootstrapError::SubscriptionError(e))?;

        debug!("Console is launched");
        Ok(Self {
            config,
            context,
            api_socket,
            sub_socket,
        })
    }

    fn wallet_create(filename: &str) -> Result<(), Error> {
        info!("Generating new HD wallet file");

        let password = rpassword::prompt_password_stderr("Password for wallet encryption: ")?;
        if !(8..256).contains(&password.len()) {
            return Err(Error::from("The length of the password must be at least 8 and no more than 256 characters"));
        }

        debug!("- collecting 64 bits of entropy");
        let master = MasterAccount::new(
            MasterKeyEntropy::Paranoid,
            Network::Bitcoin,
            password.as_str()
        ).unwrap();

        debug!("- the generated extended pubkey identifier:");
        println!("{}", master.master_public());

        let mut file = File::create(filename)?;
        file.write_all(master.encrypted())?;
        Ok(())
    }

    fn address_derive(xpubkey: ExtendedPubKey, acc_i: u32, addr_i: u32) -> Result<(), Error> {
        info!("Generating new address from account #{} and index {}", acc_i, addr_i);
        let ctx = SecpContext::new();
        let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: HD_PURPOSE})?;
        let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: HD_COIN})?;
        let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: acc_i})?;
        let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: 0})?;
        let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: addr_i})?;
        debug!("- the generated pubkey in compressed format:");
        println!("{}", Address::p2wpkh(&xpubkey.public_key, Network::Bitcoin));
        Ok(())
    }
}

#[async_trait]
impl TryService for Runtime {
    type ErrorType = tokio::task::JoinError;

    async fn try_run_loop(self) -> Result<!, Self::ErrorType> {
        loop {

        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        trace!("Shutting down sockets");
        self.api_socket.disconnect(&self.config.bpd_api)
            .unwrap_or_else(|err| error!("Error disconnecting message bus API socket: {}", err));
        self.sub_socket.disconnect(&self.config.bpd_subscr)
            .unwrap_or_else(|err| error!("Error disconnecting message bus push socket: {}", err));
    }
}
