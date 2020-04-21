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

use bitcoin::secp256k1;
use bitcoin::{TxIn, TxOut, Transaction};
use bitcoin::util::bip143;
use bitcoin::hashes::{Hash, hex::ToHex};
use bitcoin::consensus::encode;
use bitcoin::util::bip32::{self, ExtendedPrivKey, ExtendedPubKey, DerivationPath, ChildNumber};
use bitcoin::util::psbt::{self, PartiallySignedTransaction};
use bitcoin_wallet::{account::Seed, context::SecpContext};
use electrum_client as electrum;

use lnpbp::service::*;
use lnpbp::bitcoin;
use lnpbp::cmt::*;
use lnpbp::csv::network_serialize;
use lnpbp::csv::Storage;
use lnpbp::rgb::commit::Identifiable;
use lnpbp::rgb::Rgb1;
use lnpbp::rgb::{Transition, ContractId};
use rgb::fungible::invoice::SealDefinition;

use super::*;
use crate::error::Error;
use crate::data::{DepositTerminal, AssetAllocations};
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

    async fn get_deposits(&self, account_tag: &String, deposit_types: Vec<commands::bitcoin::DepositType>, offset: u32, no: u8) -> Result<Vec<DepositTerminal>, Error> {
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
                            s if s.is_p2pkh() => bitcoin::AddressType::P2pkh,
                            s if s.is_v0_p2wpkh() => bitcoin::AddressType::P2wpkh,
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

    fn get_genesis_data(&self, contract_id: ContractId) -> Result<Transition, Error> {
        let reader = self.config.data_reader(DataItem::ContractGenesis(contract_id))?;
        Ok(Transition::storage_deserialize(reader)?)
    }

    fn get_asset_allocations(&self) -> Result<AssetAllocations, Error> {
        Ok(match self.config.data_reader(DataItem::FungibleSeals) {
            Ok(mut reader) => serde_json::from_reader(reader)?,
            _ => AssetAllocations::new()
        })
    }
}

impl Runtime {
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
        let deposits = self.get_deposits(&account_tag, deposit_types, offset, no)
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

    pub fn fungible_list(self) -> Result<(), Error> {
        println!("{:^8}  |  {:^32}  |  {:^5}  |  {:^64}", "TICKER", "NAME", "SEALS", "CONTRACT ID");
        self.get_asset_allocations()?.seals
            .into_iter()
            .try_for_each(|(asset_id, allocations)| {
                let genesis = self.get_genesis_data(asset_id)?;
                if let lnpbp::rgb::metadata::Value::Str(ticker) = &genesis.meta.get(0)?.val {
                    if let lnpbp::rgb::metadata::Value::Str(title) = &genesis.meta.get(1)?.val {
                        println!("{:>8}  |  {:<32}  |  {:>5}  |  {:^64}", ticker, title, allocations.len(), asset_id);
                        Ok(())
                    } else {
                        Err(Error::Other)
                    }
                } else {
                    Err(Error::Other)
                }
            })?;
        Ok(())
    }

    pub async fn fungible_funds(mut self, account_tag: String, deposit_types: Vec<commands::bitcoin::DepositType>,
                         contract_id: ContractId, only_owned: bool) -> Result<(), Error> {
        let deposits = self
            .get_deposits(&account_tag, deposit_types, 0, 10)
            .await?;
        let deposits = deposits
            .into_iter()
            .map(|depo| {
                (depo.outpoint, depo)
            })
            .collect::<HashMap<bitcoin::OutPoint, DepositTerminal>>();

        let genesis = self.get_genesis_data(contract_id)?;
        if let lnpbp::rgb::metadata::Value::Str(ticker) = &genesis.meta.get(0)?.val {
            if let lnpbp::rgb::metadata::Value::Str(title) = &genesis.meta.get(1)?.val {
                println!("Listing {} ({}) allocations:", ticker, title);
            }
        }

        let existing_allocations = self.get_asset_allocations()?;
        let existing_allocations = existing_allocations.seals
            .get(&contract_id)
            .unwrap_or_else(|| { panic!("You do not have any spendable assets for {}", contract_id) });
        println!("{:^8}  |  {:^12}  |  {:^6}  |  {:^64}  |  {:>6}", "STATUS", "VALUE", "EXISTS", "TXID", "VOUT");
        existing_allocations.into_iter()
            .for_each(|allocation| {
                let depo = deposits.get(&allocation.seal);
                let missed = if let Some(depo) = depo {
                    depo.outpoint != allocation.seal
                } else {
                    false
                };
                println!("{:>8}  |  {:>12}  |  {:^6}  |  {:<64}  |  {:>6}",
                         if depo.is_some() {"spent"} else {"unspent"},
                         allocation.amount,
                         if missed {"no"} else {"yes"},
                         allocation.seal.txid, allocation.seal.vout);
            });

        Ok(())
    }

    pub fn fungible_issue(mut self, issue: commands::fungible::Issue) -> Result<(), Error> {
        info!("Issuing asset with parameters {}", issue);
        let allocations = issue.allocate.clone();
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

        let asset_id = genesis.transition_id()
            .expect("Probability of the commitment generation failure is less than negligible");
        let asset_id = ContractId::from_inner(asset_id.into_inner());
        println!("New asset {} is issued with ContractId={}", issue.ticker, asset_id);

        genesis.storage_serialize(self.config.data_writer(DataItem::ContractGenesis(asset_id))?)?;

        let mut assets = self.get_asset_allocations()?;
        assets.seals.insert(asset_id, allocations);
        let mut writer = self.config.data_writer(DataItem::FungibleSeals)?;
        serde_json::to_writer(writer, &assets)?;

        Ok(())
    }

    pub async fn fungible_pay(mut self, payment: commands::fungible::Pay) -> Result<(), Error> {
        const FEE: u64 = 100000;
        const DUST_LIMIT: u64 = 100000;

        // TODO: Use PSBT supplied by payee
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
         * Act 0: Know our outputs
         */
        let network = self.config.network.try_into().expect("Unsupported bitcoin network");
        let deposits = self
            .get_deposits(&payment.account, vec![commands::bitcoin::DepositType::WPKH], 0, 10)
            .await?;
        let deposits = deposits
            .into_iter()
            .map(|depo| {
                (depo.outpoint, depo)
            })
            .collect::<HashMap<bitcoin::OutPoint, DepositTerminal>>();

        /*
         * Act 1: Find asset outputs to spend
         */
        let contract_id = payment.contract_id;
        let existing_allocations = self.get_asset_allocations()?;
        let existing_allocations = existing_allocations.seals
            .get(&contract_id)
            .unwrap_or_else(|| { panic!("You do not have any spendable assets for {}", contract_id) });
        // "Coinselection"
        let required_amount = payment.amount;
        let mut found_amount = 0;
        let mut bitcoin_amount = 0;
        let mut required_bitcoins = 0;
        let seals_to_close: Vec<bitcoin::OutPoint> = existing_allocations
            .into_iter()
            .filter(|alloc| {
                deposits.get(&alloc.seal).is_some()
            })
            .filter(|alloc| {
                if found_amount < required_amount ||
                   bitcoin_amount <= required_bitcoins {
                    bitcoin_amount += deposits.get(&alloc.seal).unwrap().bitcoins.as_sat();
                    found_amount += alloc.amount;
                    true
                } else {
                    false
                }
            })
            .map(|alloc| {
                alloc.seal
            })
            .collect();
        let found_amount = found_amount;
        if found_amount < required_amount {
            panic!("You own only {} of asset, it's impossible to pay {} required by invoice", found_amount, required_amount);
        }
        if bitcoin_amount < required_bitcoins {
            panic!("We ned at least {} bitcoins to cover fees and dust limit, however found only {}", required_bitcoins, bitcoin_amount);
        }
        let fee = FEE;
        let mut bitcoins_minus_fee = bitcoin_amount - fee;
        let mut change = HashMap::new();
        // TODO: Use confidential amounts for keeping track of owned value
        let mut confidential_change = None;
        if found_amount > required_amount {
            confidential_change = Some(lnpbp::rgb::data::amount::Confidential::from(found_amount - required_amount));
            change.insert(
                // We use first output always
                0,
                confidential_change.unwrap().commitment
            );
        }

        /*
         * Act 2: Generate state transition
         */
        let mut outpoint_hash = payment.receiver;
        // TODO: Support payments to a newly generated txout
        /*match payment.receiver {
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
        };*/

        // The receiver is not accounted for in balances!
        let mut allocations = payment.allocate;
        let mut balances = rgb::fungible::allocations_to_balances(allocations);
        let confidential_amount = lnpbp::rgb::data::amount::Confidential::from(
            payment.amount // FIXME once invoices will be working: `.unwrap_or(payment.invoice.amount)`
        );

        let mut transfer = Rgb1::transfer(balances, change)?;
        let mut state = transfer.state.into_inner();
        state.push(lnpbp::rgb::state::Partial::State(lnpbp::rgb::state::Bound {
            // FIXME: Change into a proper RGB1 constant to reflect balance seal type
            id: lnpbp::rgb::seal::Type(1),
            seal: lnpbp::rgb::Seal::BlindedTxout(outpoint_hash),
            val: lnpbp::rgb::Data::Balance(confidential_amount.commitment)
        }));
        transfer.state = state.into();

        /*
         * Act 3: Generate witness transaction
         */
        let txins = seals_to_close.into_iter().map(|seal| {
            TxIn {
                previous_output: seal,
                script_sig: bitcoin::Script::new(),
                sequence: 0,
                witness: vec![]
            }
        }).collect();

        let change_box = *self.keyrings
            .get_main_keyring()
            .list_deposit_boxes(&payment.account, 0, 1)?
            .first()
            .unwrap();
        let change_address = change_box
            .get_p2wpkh_addr(bitcoin::Network::Bitcoin);
        let witness_tx = Transaction {
            version: 2,
            lock_time: 0,
            input: txins,
            output: vec![
                TxOut {
                    value: bitcoins_minus_fee,
                    script_pubkey: change_address.script_pubkey()
                }
            ]
        };

        let mut entropy = [0u8; 4];
        entropy.copy_from_slice(&contract_id[..][0..4]);
        let container = lnpbp::cmt::TxContainer {
            entropy: u32::from_be_bytes(entropy),
            fee: bitcoin::Amount::from_sat(fee),
            tx: witness_tx,
            txout_container: lnpbp::cmt::TxoutContainer::PubkeyHash(change_box.get_pubkey().key)
        };
        // TODO: Use multimessage commitment instead of transition commitment
        let tf_commitment = transfer.commitment()?;
        let tx_commitment = lnpbp::cmt::TxCommitment::commit_to(container, &tf_commitment)?;

        let mut witness_tx = tx_commitment.tx;

        // Now sign the transaction
        let secp = secp256k1::Secp256k1::new();
        let witness_tx_clone = witness_tx.clone();
        let mut hasher = bip143::SigHashCache::new(&witness_tx_clone);
        let keyring = self.keyrings
            .get_main_keyring();
        let account = keyring.get_account(&payment.account)?;
        let password = rpassword::prompt_password_stderr("Password for unlocking private key: ").unwrap();
        let mut enc = vec![];
        if let Keyring::Hierarchical { encrypted, .. } = keyring {
            enc = encrypted.clone();
        } else {
            panic!()
        }
        let encrypted = enc;
        let seed = Seed::decrypt(&encrypted, &password).expect("Wrong password");
        let xprivkey = ExtendedPrivKey::new_master(network, &seed.0).expect("Wrong password");

        /*
        Some((offset..to).map(|_| {
            let dp = dp_iter.next().unwrap();
            let sk = xprivkey.derive_priv(&secp, &dp).unwrap().private_key;
        */
        println!("{}", encode::serialize(&witness_tx).to_hex());
        for (ix, input) in witness_tx.input.iter_mut().enumerate() {
            let deposit_term = deposits.get(&input.previous_output)
                .expect("Previously found deposit terminal disappeared");
            let spent_amount = deposit_term.bitcoins.as_sat();
            let dp = account.derivation_path.clone().unwrap()
                .child(ChildNumber::Normal { index: deposit_term.derivation_index as u32 });
            let sk = xprivkey.derive_priv(&secp, &dp).unwrap().private_key;
            let seckey = sk.key;
            let pubkey = sk.public_key(&secp);
            println!("{}", sk);
            let script_sig = bitcoin::Script::new();
            let prev_script = bitcoin::Address::p2wpkh(&pubkey, network).script_pubkey();
            let hash_type = bitcoin::SigHashType::All;
            let sighash = hasher.signature_hash(ix, &prev_script, spent_amount, hash_type);
            let signature = secp.sign(&secp256k1::Message::from_slice(&sighash[..])?, &seckey).serialize_der();
            let mut with_hashtype = signature.to_vec();
            with_hashtype.push(hash_type.as_u32() as u8);
            input.witness.clear();
            input.witness.push(with_hashtype);
            input.witness.push(pubkey.to_bytes());
        }

        println!("{:?}\n\n", witness_tx);
        println!("Witness Transaction:\n{}\n\nState transition:\n{}\n\nConfidential proofs:\n{}\n\n",
                 encode::serialize(&witness_tx).to_hex(),
                 network_serialize(&transfer)?.to_hex(),
                 confidential_amount.proof);

        let socket_addr: SocketAddr = self.config.electrum_endpoint
            .try_into()
            .map_err(|_| Error::TorNotYetSupported)?;
        let mut ec = electrum::Client::new(socket_addr).await?;
        ec.transaction_broadcast(&witness_tx).await?;

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
