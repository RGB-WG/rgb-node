// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::BTreeSet;
use std::{fs, io};

use amplify::IoError;
use bitcoin::consensus;
use bitcoin::psbt::serialize::{Deserialize, Serialize};
use colored::Colorize;
use microservices::cli::LogStyle;
use microservices::shell::Exec;
use psbt::Psbt;
use rgb::blank::BlankBundle;
use rgb::psbt::{RgbExt, RgbInExt};
use rgb::{Node, StateTransfer, Transition, TransitionBundle};
use rgb_rpc::{Client, ContractValidity};
use strict_encoding::{StrictDecode, StrictEncode};

use crate::{Command, Opts};

#[derive(Debug, Display, Error, From)]
#[display(inner)]
pub enum Error {
    #[from]
    #[from(io::Error)]
    Io(IoError),

    #[from]
    Rpc(rgb_rpc::Error),

    #[from]
    StrictEncoding(strict_encoding::Error),

    #[from]
    ConsensusEncoding(consensus::encode::Error),

    #[from]
    Psbt(rgb::psbt::KeyError),

    #[from]
    Reallocation(rgb::blank::Error),
}

impl Command {
    pub fn action_string(&self) -> String {
        match self {
            Command::Register { contract, .. } => {
                format!("Registering contract {}", contract.contract_id())
            }
            Command::Contracts => s!("Listing contracts"),
            Command::State { contract_id } => format!("Quering state of {}", contract_id),
            Command::Contract { contract_id, .. } => {
                format!("Retrieving contract source for {}", contract_id)
            }
            Command::Compose { contract_id, .. } => {
                format!("Composing consignment for state transfer for contract {}", contract_id)
            }
            Command::Transfer { .. } => s!("Preparing PSBT for the state transfer"),
            Command::Finalize {
                send: Some(addr), ..
            } => format!("Finalizing state transfer and sending it to {}", addr),
            Command::Finalize { send: None, .. } => s!("Finalizing state transfer"),
        }
    }
}

impl Exec for Opts {
    type Client = Client;
    type Error = Error;

    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        if !client.hello()? {
            eprintln!("Network mismatch");
            return Ok(());
        }

        println!("{}...", self.command.action_string());

        let progress = |info| {
            println!("{}", info);
        };

        match self.command {
            Command::Register { contract, force } => {
                match client.register_contract(contract, force, progress)? {
                    ContractValidity::Valid => {
                        println!("{}: contract is valid and imported", "Success".ended())
                    }
                    ContractValidity::Invalid(status) => {
                        eprintln!("{}: contract is invalid. Detailed report:", "Error".err());
                        eprintln!("{}", serde_yaml::to_string(&status).unwrap());
                    }
                    ContractValidity::UnknownTxids(txids) => {
                        eprintln!(
                            "{}: contract is valid, but some of underlying transactions are still \
                             not mined",
                            "Warning".bold().bright_yellow()
                        );
                        eprintln!("The list of non-mined transaction ids:");
                        for txid in txids {
                            println!("- {}", txid);
                        }
                        eprintln!(
                            "{}: contract was not imported. To import the contract, re-run the \
                             command with {} argument",
                            "Warning".bold().bright_yellow(),
                            "--force".bold().bright_white(),
                        );
                    }
                }
            }
            Command::Contracts => {
                client.list_contracts()?.iter().for_each(|id| println!("{}", id));
            }
            Command::State { contract_id } => {
                let state = client.contract_state(contract_id)?;
                println!("{}", serde_yaml::to_string(&state).unwrap());
            }
            Command::Contract {
                node_types,
                contract_id,
            } => {
                let contract = client.contract(contract_id, node_types, progress)?;
                println!("{}", contract);
            }
            Command::Compose {
                node_types,
                contract_id,
                outpoints,
                output,
            } => {
                let transfer = client.consign(
                    contract_id,
                    node_types,
                    outpoints.into_iter().collect(),
                    progress,
                )?;
                println!("Saving consignment to {}", output.display());
                let file = fs::File::create(output)?;
                transfer.strict_encode(file)?;
                println!("{}", "Success".ended());
            }

            Command::Transfer {
                contract_id,
                transition,
                psbt_in,
                psbt_out,
            } => {
                // TODO: Add contracts

                let psbt_bytes = fs::read(&psbt_in)?;
                let mut psbt = Psbt::deserialize(&psbt_bytes)?;
                let transition = Transition::strict_file_load(transition)?;
                psbt.push_rgb_transition(transition)?;
                // TODO: Set consumers

                let outpoints: BTreeSet<_> =
                    psbt.inputs.iter().map(|input| input.previous_outpoint).collect();
                let state_map = client.outpoint_state(outpoints.clone(), progress)?;
                for (cid, outpoint_map) in state_map {
                    if cid == contract_id {
                        continue;
                    }
                    let blank_bundle = TransitionBundle::blank(&outpoint_map, &bmap! {})?;
                    for (transition, indexes) in blank_bundle.revealed_iter() {
                        psbt.push_rgb_transition(transition.clone())?;
                        for no in indexes {
                            psbt.inputs[*no as usize]
                                .set_rgb_consumer(contract_id, transition.node_id())?;
                        }
                    }
                }

                let psbt_bytes = psbt.serialize();
                fs::write(psbt_out.unwrap_or(psbt_in), psbt_bytes)?;
            }

            Command::Finalize {
                psbt: psbt_path,
                consignment_in,
                consignment_out,
                endseals,
                send,
                // TODO: Add PSBT out
            } => {
                let psbt_bytes = fs::read(&psbt_path)?;
                let psbt = Psbt::deserialize(&psbt_bytes)?;
                let consignment = StateTransfer::strict_file_load(&consignment_in)?;
                let transfer = client.transfer(consignment, endseals, psbt, send, progress)?;
                // TODO: Call tapret_finalize on PSBT and save PSBT
                transfer.strict_file_save(consignment_out.unwrap_or(consignment_in))?;

                // TODO: Register disclosure with the client
            }
        }

        Ok(())
    }
}
