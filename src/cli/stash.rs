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

use lnpbp::rgb::{ContractId, SchemaId, ToBech32};

use crate::api::Reply;
use crate::cli::{Error, OutputFormat, Runtime};

#[derive(Clap, Clone, Debug, Display)]
#[display(Debug)]
pub enum SchemaCommand {
    /// Lists all known schemata
    List {
        /// Format for information output
        #[clap(short, long, arg_enum, default_value = "yaml")]
        format: OutputFormat,
    },

    /// Export schema data
    Export {
        /// Format for information output
        #[clap(short, long, arg_enum, default_value = "yaml")]
        format: OutputFormat,

        #[clap()]
        schema_id: SchemaId,
    },
}

#[derive(Clap, Clone, Debug, Display)]
#[display(Debug)]
pub enum GenesisCommand {
    /// Lists all known contract ids
    List {
        /// Format for information output
        #[clap(short, long, arg_enum, default_value = "yaml")]
        format: OutputFormat,
    },

    /// Export schema data
    Export {
        /// Format for information output
        #[clap(short, long, arg_enum, default_value = "yaml")]
        format: OutputFormat,

        #[clap()]
        contract_id: ContractId,
    },
}

impl SchemaCommand {
    pub fn exec(self, runtime: Runtime) -> Result<(), Error> {
        match self {
            SchemaCommand::List { format } => self.exec_list(runtime, format),
            SchemaCommand::Export { format, schema_id } => {
                self.exec_export(runtime, format, schema_id)
            }
        }
    }

    fn exec_list(&self, mut runtime: Runtime, format: OutputFormat) -> Result<(), Error> {
        match &*runtime.list_schemata()? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::SchemaIds(ids) => {
                let output = match format {
                    OutputFormat::Yaml => serde_yaml::to_string(&ids)?,
                    OutputFormat::Json => serde_json::to_string(&ids)?,
                    OutputFormat::Toml => toml::to_string(&ids)?,
                    _ => Err(Error::FormatNotSupported)?,
                };
                println!("{}", output);
            }
            _ => {
                eprintln!(
                    "Unexpected server error; probably you connecting with outdated client version"
                );
            }
        }
        Ok(())
    }

    fn exec_export(
        &self,
        mut runtime: Runtime,
        format: OutputFormat,
        schema_id: SchemaId,
    ) -> Result<(), Error> {
        match &*runtime.schema(schema_id)? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::Schema(schema) => {
                let output = match format {
                    OutputFormat::Yaml => serde_yaml::to_string(&schema)?,
                    OutputFormat::Json => serde_json::to_string(&schema)?,
                    OutputFormat::Bech32 => schema.to_bech32_string(),
                    _ => Err(Error::FormatNotSupported)?,
                };
                println!("{}", output);
            }
            _ => {
                eprintln!(
                    "Unexpected server error; probably you connecting with outdated client version"
                );
            }
        }
        Ok(())
    }
}

impl GenesisCommand {
    pub fn exec(self, runtime: Runtime) -> Result<(), Error> {
        match self {
            GenesisCommand::List { format } => self.exec_list(runtime, format),
            GenesisCommand::Export {
                format,
                contract_id,
            } => self.exec_export(runtime, format, contract_id),
        }
    }

    fn exec_list(&self, mut runtime: Runtime, format: OutputFormat) -> Result<(), Error> {
        match &*runtime.list_geneses()? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::ContractIds(ids) => {
                let output = match format {
                    OutputFormat::Yaml => serde_yaml::to_string(&ids)?,
                    OutputFormat::Json => serde_json::to_string(&ids)?,
                    OutputFormat::Toml => toml::to_string(&ids)?,
                    _ => Err(Error::FormatNotSupported)?,
                };
                println!("{}", output);
            }
            _ => {
                eprintln!(
                    "Unexpected server error; probably you connecting with outdated client version"
                );
            }
        }
        Ok(())
    }

    fn exec_export(
        &self,
        mut runtime: Runtime,
        format: OutputFormat,
        contract_id: ContractId,
    ) -> Result<(), Error> {
        match &*runtime.genesis(contract_id)? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::Genesis(genesis) => {
                let output = match format {
                    OutputFormat::Yaml => serde_yaml::to_string(&genesis)?,
                    OutputFormat::Json => serde_json::to_string(&genesis)?,
                    OutputFormat::Bech32 => genesis.to_bech32_string(),
                    _ => Err(Error::FormatNotSupported)?,
                };
                println!("{}", output);
            }
            _ => {
                eprintln!(
                    "Unexpected server error; probably you connecting with outdated client version"
                );
            }
        }
        Ok(())
    }
}
