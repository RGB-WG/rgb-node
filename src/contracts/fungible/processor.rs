// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use chrono::Utc;
use core::convert::TryFrom;
use serde::Deserialize;
use std::collections::BTreeMap;

use lnpbp::bitcoin::util::psbt::PartiallySignedTransaction;
use lnpbp::bp;
use lnpbp::rgb::prelude::*;

use super::schema::{self, AssignmentsType, FieldType};
use super::{Asset, Coins, Outcoins};

use crate::error::{BootstrapError, ServiceErrorDomain};
use crate::util::SealSpec;
use crate::{field, type_map};

pub struct Processor {}

#[derive(Debug, Deserialize)]
pub enum IssueStructure {
    SingleIssue,
    MultipleIssues {
        max_supply: f32,
        reissue_control: SealSpec,
    },
}

impl Processor {
    pub fn new() -> Result<Self, BootstrapError> {
        debug!("Instantiating RGB asset manager ...");

        let me = Self {};
        /*
        let storage = rgb_storage.clone();
        let me = Self {
            rgb_storage,
            asset_storage,
        };
         */
        let _schema = schema::schema();
        //if !me.rgb_storage.lock()?.has_schema(schema.schema_id())? {
        info!("RGB fungible assets schema file not found, creating one");
        //storage.lock()?.add_schema(&schema)?;
        //}

        Ok(me)
    }

    pub fn issue(
        &mut self,
        network: bp::Network,
        ticker: String,
        name: String,
        description: Option<String>,
        issue_structure: IssueStructure,
        allocations: Vec<Outcoins>,
        precision: u8,
        prune_seals: Vec<SealSpec>,
        dust_limit: Option<Amount>,
    ) -> Result<(Asset, Genesis), ServiceErrorDomain> {
        let now = Utc::now().timestamp();
        let mut metadata = type_map! {
            FieldType::Ticker => field!(String, ticker),
            FieldType::Name => field!(String, name),
            FieldType::FractionalBits => field!(U8, precision),
            FieldType::DustLimit => field!(U64, dust_limit.unwrap_or(0)),
            FieldType::Timestamp => field!(U32, now as u32)
        };
        if let Some(description) = description {
            metadata.insert(-FieldType::Description, field!(String, description));
        }

        let mut issued_supply = 0u64;
        let allocations = allocations
            .into_iter()
            .map(|outcoins| {
                let amount = Coins::transmutate(outcoins.coins, precision);
                issued_supply += amount;
                (outcoins.seal_definition(), amount)
            })
            .collect();
        let mut assignments = BTreeMap::new();
        assignments.insert(
            -AssignmentsType::Assets,
            AssignmentsVariant::zero_balanced(allocations, 0),
        );
        metadata.insert(-FieldType::IssuedSupply, field!(U64, issued_supply));

        let mut total_supply = issued_supply;
        if let IssueStructure::MultipleIssues {
            max_supply,
            reissue_control,
        } = issue_structure
        {
            total_supply = Coins::transmutate(max_supply, precision);
            if total_supply < issued_supply {
                Err(ServiceErrorDomain::Schema(format!(
                    "Total supply ({}) should be greater than the issued supply ({})",
                    total_supply, issued_supply
                )))?;
            }
            metadata.insert(-FieldType::TotalSupply, field!(U64, total_supply));
            assignments.insert(
                -AssignmentsType::Issue,
                AssignmentsVariant::Void(bset![Assignment::Revealed {
                    seal_definition: reissue_control.seal_definition(),
                    assigned_state: data::Void
                }]),
            );
        } else {
            metadata.insert(-FieldType::TotalSupply, field!(U64, total_supply));
        }

        assignments.insert(
            -AssignmentsType::Prune,
            AssignmentsVariant::Void(
                prune_seals
                    .into_iter()
                    .map(|seal_spec| Assignment::Revealed {
                        seal_definition: seal_spec.seal_definition(),
                        assigned_state: data::Void,
                    })
                    .collect(),
            ),
        );

        let genesis = Genesis::with(
            schema::schema().schema_id(),
            network,
            metadata,
            assignments,
            vec![],
        );

        let asset = Asset::try_from(genesis.clone())?;
        //self.asset_storage.lock()?.add_asset(asset.clone())?;

        Ok((asset, genesis))
    }

    pub fn transfer(
        &mut self,
        psbt: &mut PartiallySignedTransaction,
        allocate: Vec<Outcoins>,
        amount: Amount,
        contract_id: ContractId,
        receiver: bp::blind::OutpointHash,
    ) -> Result<() /*Consignment*/, ServiceErrorDomain> {
        /*
        const FEE: u64 = 100000;
        const DUST_LIMIT: u64 = 100000;

        /*
         * Act 0: Know our outputs
         */
        let network = self
            .config
            .network
            .try_into()
            .expect("Unsupported bitcoin network");
        let deposits = self
            .get_deposits(
                &payment.account,
                vec![commands::bitcoin::DepositType::WPKH],
                0,
                10,
            )
            .await?;
        let deposits = deposits
            .into_iter()
            .map(|depo| (depo.outpoint, depo))
            .collect::<HashMap<bitcoin::OutPoint, DepositTerminal>>();

        /*
         * Act 1: Find asset outputs to spend
         */
        let contract_id = payment.contract_id;
        let existing_allocations = self.get_asset_allocations()?;
        let existing_allocations = existing_allocations
            .seals
            .get(&contract_id)
            .unwrap_or_else(|| panic!("You do not have any spendable assets for {}", contract_id));
        // "Coinselection"
        let required_amount = payment.amount;
        let mut found_amount = 0;
        let mut bitcoin_amount = 0;
        let mut required_bitcoins = 0;
        let seals_to_close: Vec<bitcoin::OutPoint> = existing_allocations
            .into_iter()
            .filter(|alloc| deposits.get(&alloc.seal).is_some())
            .filter(|alloc| {
                if found_amount < required_amount || bitcoin_amount <= required_bitcoins {
                    bitcoin_amount += deposits.get(&alloc.seal).unwrap().bitcoins.as_sat();
                    found_amount += alloc.amount;
                    true
                } else {
                    false
                }
            })
            .map(|alloc| alloc.seal)
            .collect();
        let found_amount = found_amount;
        if found_amount < required_amount {
            panic!(
                "You own only {} of asset, it's impossible to pay {} required by invoice",
                found_amount, required_amount
            );
        }
        if bitcoin_amount < required_bitcoins {
            panic!(
                "We ned at least {} bitcoins to cover fees and dust limit, however found only {}",
                required_bitcoins, bitcoin_amount
            );
        }
        let fee = FEE;
        let mut bitcoins_minus_fee = bitcoin_amount - fee;
        let mut change = HashMap::new();
        // TODO: Use confidential amounts for keeping track of owned value
        let mut confidential_change = None;
        if found_amount > required_amount {
            confidential_change = Some(lnpbp::rgb::data::amount::Confidential::from(
                found_amount - required_amount,
            ));
            change.insert(
                // We use first output always
                0,
                confidential_change.unwrap().commitment,
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
            payment.amount, // FIXME once invoices will be working: `.unwrap_or(payment.invoice.amount)`
        );

        let mut transfer = Rgb1::transfer(balances, change)?;
        let mut state = transfer.state.into_inner();
        state.push(lnpbp::rgb::state::Partial::State(
            lnpbp::rgb::state::Bound {
                // FIXME: Change into a proper RGB1 constant to reflect balance seal type
                id: lnpbp::rgb::seal::Type(1),
                seal: lnpbp::rgb::Seal::BlindedTxout(outpoint_hash),
                val: lnpbp::rgb::Data::Balance(confidential_amount.commitment),
            },
        ));
        transfer.state = state.into();

        /*
         * Act 3: Generate witness transaction
         */
        let txins = seals_to_close
            .into_iter()
            .map(|seal| TxIn {
                previous_output: seal,
                script_sig: bitcoin::Script::new(),
                sequence: 0,
                witness: vec![],
            })
            .collect();

        let change_box = *self
            .keyrings
            .get_main_keyring()
            .list_deposit_boxes(&payment.account, 0, 1)?
            .first()
            .unwrap();
        let change_address = change_box.get_p2wpkh_addr(bitcoin::Network::Bitcoin);
        let witness_tx = Transaction {
            version: 2,
            lock_time: 0,
            input: txins,
            output: vec![TxOut {
                value: bitcoins_minus_fee,
                script_pubkey: change_address.script_pubkey(),
            }],
        };

        let mut entropy = [0u8; 4];
        entropy.copy_from_slice(&contract_id[..][0..4]);
        let container = lnpbp::cmt::TxContainer {
            entropy: u32::from_be_bytes(entropy),
            fee: bitcoin::Amount::from_sat(fee),
            tx: witness_tx,
            txout_container: lnpbp::cmt::TxoutContainer::PubkeyHash(change_box.get_pubkey().key),
        };
        // TODO: Use multimessage commitment instead of transition commitment
        let tf_commitment = transfer.commitment()?;
        let tx_commitment = lnpbp::cmt::TxCommitment::commit_to(container, &tf_commitment)?;

        let mut witness_tx = tx_commitment.tx;

        // Now sign the transaction
        let secp = secp256k1::Secp256k1::new();
        let witness_tx_clone = witness_tx.clone();
        let mut hasher = bip143::SigHashCache::new(&witness_tx_clone);
        let keyring = self.keyrings.get_main_keyring();
        let account = keyring.get_account(&payment.account)?;
        let password =
            rpassword::prompt_password_stderr("Password for unlocking private key: ").unwrap();
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
            let deposit_term = deposits
                .get(&input.previous_output)
                .expect("Previously found deposit terminal disappeared");
            let spent_amount = deposit_term.bitcoins.as_sat();
            let dp = account
                .derivation_path
                .clone()
                .unwrap()
                .child(ChildNumber::Normal {
                    index: deposit_term.derivation_index as u32,
                });
            let sk = xprivkey.derive_priv(&secp, &dp).unwrap().private_key;
            let seckey = sk.key;
            let pubkey = sk.public_key(&secp);
            println!("{}", sk);
            let script_sig = bitcoin::Script::new();
            let prev_script = bitcoin::Address::p2wpkh(&pubkey, network).script_pubkey();
            let hash_type = bitcoin::SigHashType::All;
            let sighash = hasher.signature_hash(ix, &prev_script, spent_amount, hash_type);
            let signature = secp
                .sign(&secp256k1::Message::from_slice(&sighash[..])?, &seckey)
                .serialize_der();
            let mut with_hashtype = signature.to_vec();
            with_hashtype.push(hash_type.as_u32() as u8);
            input.witness.clear();
            input.witness.push(with_hashtype);
            input.witness.push(pubkey.to_bytes());
        }

        println!("{:?}\n\n", witness_tx);
        println!(
            "Witness Transaction:\n{}\n\nState transition:\n{}\n\nConfidential proofs:\n{}\n\n",
            encode::serialize(&witness_tx).to_hex(),
            network_serialize(&transfer)?.to_hex(),
            confidential_amount.proof
        );

        let socket_addr: SocketAddr = self
            .config
            .electrum_endpoint
            .try_into()
            .map_err(|_| Error::TorNotYetSupported)?;
        let mut ec = electrum::Client::new(socket_addr).await?;
        ec.transaction_broadcast(&witness_tx).await?;

         */
        Ok(())
    }

    /*
    pub fn assets(&self) -> Result<Vec<Asset>, InteroperableError> {
        Ok(self
            .asset_storage
            .lock()?
            .assets()?
            .into_iter()
            .map(Asset::clone)
            .collect())
    }

    pub fn import(&self, data: ExchangableData) -> Result<MagicNumber, InteroperableError> {
        unimplemented!()
    }

    pub fn pay(&self, invoice: Invoice) -> Result<(), InteroperableError> {
        let assets = self.asset_storage.lock()?.assets()?;
    }
     */

    /*
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

     */
}
