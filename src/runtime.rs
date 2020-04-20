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


use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::net::SocketAddr;

use bitcoin::Transaction;
use bitcoin::util::bip32::{DerivationPath, ChildNumber};
use bitcoin::util::psbt::{self, PartiallySignedTransaction};
use electrum_client as electrum;

use lnpbp::service::*;
use lnpbp::bitcoin;
use lnpbp::bp;
use lnpbp::csv::Storage;
use lnpbp::rgb::commit::Identifiable;
use lnpbp::rgb::data::amount;
use lnpbp::rgb::Rgb1;
use rgb::fungible::invoice::SealDefinition;

use super::*;
use crate::error::Error;
use crate::data::{DepositTerminal, SpendingStructure};
use crate::accounts::{KeyringManager, Account};


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

    async fn get_deposits(&self, account_tag: String, deposit_types: Vec<commands::bitcoin::DepositType>, offset: u32, no: u8) -> Result<Vec<DepositTerminal>, Error> {
        use commands::bitcoin::DepositType::*;

        let socket_addr: SocketAddr = self.config.electrum_endpoint
            .try_into()
            .map_err(|_| Error::TorNotYetSupported)?;
        let mut ec = electrum::Client::new(socket_addr).await?;

        let index = offset;
        let network: bitcoin::Network = self.config.network.try_into().unwrap_or(bitcoin::Network::Testnet).into();
        let keyring = self.keyrings
            .get_main_keyring();
        let account = keyring.get_account(&account_tag)?;
        let base_derivation = account.derivation_path?;
        let depo_boxes = keyring
            .list_deposit_boxes(&account_tag, offset, no)
            .ok_or(Error::AccountNotFound)?;
        let data: HashMap<bitcoin::Script, usize> = depo_boxes
            .iter()
            .enumerate()
            .map(|(idx, depo)| -> Vec<(bitcoin::Script, usize)> {
                let mut s = vec![];
                if deposit_types.contains(&PKH) { s.push(depo.get_p2pkh_addr(network)) }
                if deposit_types.contains(&WPKH) { s.push(depo.get_p2wpkh_addr(network)) }
                s.into_iter().map(|addr| -> (bitcoin::Script, usize) {
                    (addr.clone().payload
                        .script_pubkey()
                        .into_script(),
                    idx)
                }).collect()
            })
            .flatten()
            .collect();

        let res = ec.batch_script_list_unspent(data.keys()).await?;

        Ok(data.into_iter().zip(res.into_iter())
            .map(|((script, index), list)| {
                if list.is_empty() { return vec![] }
                list.into_iter().map(|info| {
                    DepositTerminal {
                        outpoint: bitcoin::OutPoint { txid: info.tx_hash, vout: info.tx_pos as u32 },
                        derivation_index: index,
                        spending_structure: match &script {
                            s if s.is_p2pkh() => SpendingStructure::P2PKH,
                            s if s.is_v0_p2wpkh() => SpendingStructure::P2WPKH,
                            _ => panic!("Unknown spending structure"),
                        },
                        bitcoins: bitcoin::Amount::from_sat(info.value),
                        fungibles: Default::default()
                    }
                })
                .collect()
            })
            .flatten()
            .collect())
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
            .list_deposit_boxes(&account_tag, offset, no)
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
        info!("Listing bitcoin funds");
        let deposits = self.get_deposits(account_tag, deposit_types, offset, no)
            .await?;

        println!("{:>4}:   {:^16}  |  {:^64}  |  {:^5}  |  {}", "ID", "VALUE", "TXID", "VOUT", "TYPE");
        deposits
            .into_iter()
            .for_each(|depo| {
                let btc = format!("{}", depo.bitcoins);
                println!("{:>4}:   {:>16}  |  {:64}  |  {:>5}  |  {}",
                         depo.derivation_index,
                         btc,
                         depo.outpoint.txid, depo.outpoint.vout, depo.spending_structure);
            });

        Ok(())
    }

    pub fn fungible_list(mut self, only_owned: bool) -> Result<(), Error> {

    }

    pub fn fungible_issue(mut self, issue: commands::fungible::Issue) -> Result<(), Error> {
        info!("Issuing asset with parameters {}", issue);
        let balances = rgb::fungible::allocations_to_balances(issue.allocate);
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

    pub fn fungible_pay(mut self, payment: commands::fungible::Pay) -> Result<(), Error> {
        let mut psbt = PartiallySignedTransaction {
            // TODO: Replace with Transaction::default when the new version of
            //       bitcoin crate is released
            global: psbt::Global { unsigned_tx: Transaction {
                version: 0,
                lock_time: 0,
                input: vec![],
                output: vec![]
            }, unknown: BTreeMap::new() },
            inputs: vec![],
            outputs: vec![]
        };

        /*
         * Act 1: Generate state transition
         */
        let mut seal = match payment.invoice.assign_to {
            SealDefinition::NewUtxo(supplied_psbt, vout) => {
                // According to BIP-174, PSBT provided by creator must not contain
                // non-transactional input or output fields
                if !psbt.inputs.is_empty() || !psbt.outputs.is_empty() {
                    return Err(Error::WrongInvoicePsbtStructure)
                }
                psbt = supplied_psbt;
                lnpbp::rgb::Seal::WitnessTxout(vout)
            },
            SealDefinition::ExistingUtxo(blind_outpoint) =>
                lnpbp::rgb::Seal::BlindedTxout(blind_outpoint),
        };

        // The receiver is not accounted for in balances!
        let balances = rgb::fungible::allocations_to_balances(payment.allocate);
        let confidential_amount = lnpbp::rgb::data::amount::Confidential::from(
            payment.amount.unwrap_or(payment.invoice.amount)
        );

        let mut transfer = Rgb1::transfer(balances)?;
        let mut state = transfer.state.into_inner();
        state.push(lnpbp::rgb::state::Partial::State(lnpbp::rgb::state::Bound {
            // FIXME: Change into a proper RGB1 constant to reflect balance seal type
            id: lnpbp::rgb::seal::Type(1),
            seal: seal,
            val: lnpbp::rgb::Data::Balance(confidential_amount.commitment)
        }));
        transfer.state = state.into();

        /*
         * Act 2: Find asset outputs to spend
         */


        /*
         * Act 3: Generate witness transaction
         */

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
