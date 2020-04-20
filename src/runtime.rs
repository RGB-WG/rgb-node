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


use std::convert::TryInto;

use lnpbp::service::*;
use lnpbp::bitcoin;
use bitcoin::util::bip32::{DerivationPath};
use electrum_client as electrum;

use crate::lnpbp::rgb::commit::Identifiable;
use lnpbp::rgb::data::amount;
use lnpbp::rgb::Rgb1;

use super::*;
use crate::error::Error;
use crate::accounts::{KeyringManager, Account};
use lnpbp::csv::Storage;
use std::net::SocketAddr;


pub struct Runtime {
    config: Config,
    context: zmq::Context,
    api_socket: zmq::Socket,
    sub_socket: zmq::Socket,
    keyrings: KeyringManager,
}

impl Runtime {
    pub async fn init(config: Config) -> Result<Self, Error> {
        let context = zmq::Context::new();

        debug!("Opening API socket to bpd on {} ...", config.bpd_api);
        let api_socket = context.socket(zmq::REQ)
            .map_err(|e| Error::PublishingError(e))?;
        api_socket.bind(&config.bpd_api)
            .map_err(|e| Error::PublishingError(e))?;

        debug!("Subscribing to bpd notifications on {} ...", config.bpd_subscr);
        let sub_socket = context.socket(zmq::SUB)
            .map_err(|e| Error::SubscriptionError(e))?;
        sub_socket.connect(&config.bpd_subscr)
            .map_err(|e| Error::SubscriptionError(e))?;
        sub_socket.set_subscribe("".as_bytes())
            .map_err(|e| Error::SubscriptionError(e))?;

        debug!("Opening vault in safe mode");
        if !config.data_path(DataItem::KeyringVault).exists() {
            Err(Error::from("Data directory does not exist: \
                    wrong configuration or you have not initialized data directory \
                    Try running `kaleidoscope init`"))?;
        }
        let keyrings = KeyringManager::load(config.data_path(DataItem::KeyringVault))?;

        debug!("Initialization is completed");
        Ok(Self {
            config,
            context,
            api_socket,
            sub_socket,
            keyrings,
        })
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

impl Runtime {
    fn get_keyring(&mut self) -> &mut Keyring {
        self.keyrings
            .keyrings
            .get_mut(0)
            .expect("Keyring manager contains no accounts")
    }

    pub fn account_list(self) -> Result<(), Error> {
        info!("Listing known accounts");
        println!("{}", self.keyrings);
        Ok(())
    }

    pub fn account_create(mut self, name: String, derivation_path: DerivationPath, description: String) -> Result<(), Error> {
        let mut keyring = self.get_keyring();
        info!("Creating new account {} with derivation path {}", name, derivation_path);
        keyring.add_account(Account {
            name: name.clone(),
            description,
            derivation_path: Some(derivation_path)
        })?;
        debug!("Saving into the vault");
        self.keyrings.store(self.config.data_path(DataItem::KeyringVault))?;
        println!("New account {} successfully added to the default keyring and saved to the vault", name);
        Ok(())
    }

    pub fn account_deposit_boxes(self, account_tag: String, offset: u32, no: u8) -> Result<(), Error> {
        info!("Listing deposit boxes");
        let mut index = offset;
        let network = self.config.network.try_into().unwrap_or(bitcoin::Network::Testnet);
        println!("{:>4}:  {:64}    {:32}    {:48}", "ID", "PUBKEY", "P2WPKH ADDRESS", "P2PKH ADDRESS");
        self.keyrings
            .get_main_keyring()
            .list_deposit_boxes(account_tag, offset, no)
            .ok_or(Error::AccountNotFound)?
            .iter()
            .for_each(|depo| {
                println!("{:>4}:  {:64}    {:32}    {:48}",
                         index, depo.get_pubkey(), depo.get_p2wpkh_addr(network), depo.get_p2pkh_addr(network));
                index += 1;
            });
        Ok(())
    }

    pub async fn bitcoin_funds(self, account_tag: String, deposit_types: Vec<commands::bitcoin::DepositType>, offset: u32, no: u8) -> Result<(), Error> {
        use commands::bitcoin::DepositType::*;
        info!("Listing bitcoin funds");

        let socket_addr: SocketAddr = self.config.electrum_endpoint
            .try_into()
            .map_err(|_| Error::TorNotYetSupported)?;
        let mut ec = electrum::Client::new(socket_addr).await?;

        let index = offset;
        let network: bitcoin::Network = self.config.network.try_into().unwrap_or(bitcoin::Network::Testnet).into();
        let depo_boxes = self.keyrings
            .get_main_keyring()
            .list_deposit_boxes(account_tag, offset, no)
            .ok_or(Error::AccountNotFound)?;
        let req: Vec<(bitcoin::Address, bitcoin::Script)> = depo_boxes
            .iter()
            .map(|depo| -> Vec<(bitcoin::Address, bitcoin::Script)> {
                let mut s = vec![];
                if deposit_types.contains(&PKH) { s.push(depo.get_p2pkh_addr(network)) }
                if deposit_types.contains(&WPKH) { s.push(depo.get_p2wpkh_addr(network)) }
                s.into_iter().map(|addr| {
                    (addr.clone(),
                     addr.payload
                        .script_pubkey()
                        .into_script())
                }).collect()
            })
            .flatten()
            .collect();

        let (addresses, scripts): (Vec<_>, Vec<bitcoin::Script>) = req.into_iter().unzip();
        let scripts = scripts.iter().collect::<Vec<&bitcoin::Script>>();
        let res = ec.batch_script_list_unspent(scripts).await?;

        addresses.into_iter().zip(res.into_iter())
            .for_each(|(addr, list)| {
                if list.is_empty() { return; }
                println!(" {}:", addr);
                println!(" ------------------------------------------------------------------------------------------------------------------------- ");
                list.into_iter().for_each(|item| {
                    println!("\t{:4.6} BTC  |  {:64}:{:<5}  |  height: {:>6}",
                             (item.value as f32) / 100_000_000.0,
                             item.tx_hash, item.tx_pos, item.height);
                });
            });

        Ok(())
    }

    pub fn fungible_issue(mut self, issue: commands::fungible::Issue) -> Result<(), Error> {
        info!("Issuing asset with parameters {}", issue);
        let balances = issue.allocate.iter().map(|alloc| {
            let confidential = amount::Confidential::from(alloc.amount);
            (alloc.seal, confidential.commitment)
        }).collect();
        let genesis = Rgb1::issue(
            self.config.network,
            &issue.ticker,
            &issue.title,
            issue.description.as_deref(),
            balances,
            issue.precision,
            issue.supply,
            issue.dust_limit
        )?;

        let asset_id = genesis.commitment()
            .expect("Probability of the commitment generation failure is less than negligible");
        println!("New asset {} is issued with ContractId={}", issue.ticker, asset_id);

        genesis.storage_serialize(self.config.data_writer(DataItem::ContractGenesis(asset_id))?)?;

        Ok(())
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
