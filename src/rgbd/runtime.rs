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

#[cfg(any(feature = "node"))]
use clap::Clap;
#[cfg(any(feature = "node"))]
use futures::future::join_all;
use tokio::{process, task};

use super::Config;
#[cfg(any(feature = "node"))]
use crate::contracts::fungible;
use crate::error::{BootstrapError, RuntimeError};
#[cfg(any(feature = "node"))]
use crate::service::TryService;
#[cfg(feature = "node")]
use crate::stash;

pub struct Runtime {
    config: Config,
}

impl Runtime {
    pub async fn init(config: Config) -> Result<Self, BootstrapError> {
        Ok(Self { config })
    }

    #[cfg(any(feature = "node"))]
    fn get_task_for(
        name: &str,
        args: &[String],
    ) -> Result<task::JoinHandle<Result<(), DaemonError>>, DaemonError> {
        match name {
            "stashd" => {
                let opts = stash::Opts::parse_from(args.into_iter());
                Ok(task::spawn(async move {
                    Ok(stash::main_with_config(opts.into()).await?)
                }))
            }
            "fungibled" => {
                let opts = fungible::Opts::parse_from(args.into_iter());
                Ok(task::spawn(async move {
                    Ok(fungible::main_with_config(opts.into()).await?)
                }))
            }
            _ => Err(DaemonError::UnknownDaemon(name.into())),
        }
    }

    #[cfg(any(feature = "node"))]
    fn daemon(&self, bin: &str) -> Result<DaemonHandle, DaemonError> {
        let common_args: Vec<String> = vec![
            s!("-v"), // required flag but doesn't change verbosity
            s!("--data-dir"),
            self.config
                .data_dir
                .to_str()
                .expect("Datadir path is wrong")
                .to_string(),
            s!("--network"),
            self.config.network.to_string(),
        ];
        let mut fungibled_args: Vec<String> = common_args.clone();
        let mut stashd_args: Vec<String> = common_args.clone();
        fungibled_args.extend(
            vec![
                s!("--rpc"),
                self.config.fungible_rpc_endpoint.to_string(),
                s!("--pub"),
                self.config.fungible_pub_endpoint.to_string(),
                s!("--stash-rpc"),
                self.config.stash_rpc_endpoint.to_string(),
                s!("--stash-sub"),
                self.config.stash_pub_endpoint.to_string(),
                s!("--cache"),
                self.config.cache.to_string(),
                s!("--format"),
                self.config.format.to_string(),
            ]
            .iter()
            .cloned(),
        );
        stashd_args.extend(vec![
            s!("--rpc"),
            self.config.stash_rpc_endpoint.to_string(),
            s!("--pub"),
            self.config.stash_pub_endpoint.to_string(),
            s!("--stash"),
            self.config.stash.to_string(),
            s!("--index"),
            self.config.index.to_string(),
            s!("--bind"),
            self.config.p2p_endpoint.to_string(),
            s!("--electrum"),
            self.config.electrum_server.to_string(),
        ]);
        let args;
        match bin {
            "stashd" => {
                args = stashd_args;
            }
            "fungibled" => {
                args = fungibled_args;
            }
            _ => args = [].to_vec(),
        }

        if self.config.threaded {
            Ok(DaemonHandle::Task(Self::get_task_for(bin, &args)?))
        } else {
            let mut daemon = self.config.bin_dir.clone();
            daemon.push(bin);
            let mut cmd = process::Command::new(daemon);
            cmd.args(&args);
            Ok(DaemonHandle::Process(cmd.spawn()?))
        }
    }
}

#[derive(Debug)]
enum DaemonHandle {
    Process(process::Child),
    Task(task::JoinHandle<Result<(), DaemonError>>),
}

#[derive(Debug, Error)]
pub enum DaemonError {
    Process(RuntimeError),
    Task(task::JoinError),
    IO(std::io::Error),
    Bootstrap(BootstrapError),
    UnknownDaemon(String),
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<RuntimeError> for DaemonError {
    fn from(other: RuntimeError) -> DaemonError {
        DaemonError::Process(other)
    }
}

impl From<std::io::Error> for DaemonError {
    fn from(other: std::io::Error) -> DaemonError {
        DaemonError::IO(other)
    }
}

impl From<task::JoinError> for DaemonError {
    fn from(other: task::JoinError) -> DaemonError {
        DaemonError::Task(other)
    }
}

impl From<BootstrapError> for DaemonError {
    fn from(other: BootstrapError) -> DaemonError {
        DaemonError::Bootstrap(other)
    }
}

impl DaemonHandle {
    async fn future(self) -> Result<(), DaemonError> {
        match self {
            DaemonHandle::Process(mut child) => Ok(child.wait().await.map(|_| ())?),
            DaemonHandle::Task(task) => Ok(task.await??),
        }
    }
}

#[cfg(any(feature = "node"))]
#[async_trait]
impl TryService for Runtime {
    type ErrorType = DaemonError;

    async fn try_run_loop(mut self) -> Result<(), DaemonError> {
        let mut handlers = vec![];

        handlers.push(self.daemon("stashd")?);

        self.config.contracts.iter().try_for_each(
            |contract_name| -> Result<(), DaemonError> {
                handlers.push(self.daemon(contract_name.daemon_name())?);
                Ok(())
            },
        )?;

        join_all(handlers.into_iter().map(|d| d.future()))
            .await
            .into_iter()
            .try_for_each(|res| -> Result<(), DaemonError> {
                res?;
                Ok(())
            })?;

        unreachable!()
    }
}

#[cfg(any(feature = "node"))]
pub async fn main_with_config(config: Config) -> Result<(), BootstrapError> {
    let runtime = Runtime::init(config).await?;
    runtime.run_or_panic("RGBd runtime").await;

    unreachable!()
}
