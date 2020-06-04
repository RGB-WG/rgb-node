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

use futures::future::join_all;
use tokio::{process, task};

use clap::derive::Clap;

use lnpbp::TryService;

use super::Config;
use crate::error::{BootstrapError, RuntimeError};

use crate::stash::Opts as StashOpts;
use crate::stash::main_with_opts as stash_main;

pub struct Runtime {
    config: Config,
}

impl Runtime {
    pub async fn init(config: Config) -> Result<Self, BootstrapError> {
        Ok(Self { config })
    }

    fn get_task_for(name: &str, args: &[&str]) -> Result<task::JoinHandle<()>, DaemonError> {
        match name {
            "stashd" => {
                let opts = StashOpts::parse_from(args.into_iter());
                Ok(task::spawn(async move {
                    // TODO: errors should be Sync so that they can be sent between threads
                    stash_main(opts).await;
                }))
            },
            _ => Err(DaemonError::UnknownDaemon)
        }
    }

    fn daemon(&self, bin: &str) -> Result<DaemonHandle, DaemonError> {
        let args = [
                "-v",
                "-v",
                "-v",
                "-v",
                "--data-dir",
                self.config.data_dir.to_str().expect("Binary path is wrong"),
            ];

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
    Task(task::JoinHandle<()>),
}

#[derive(Debug)]
enum DaemonError {
    Process(RuntimeError),
    Task(task::JoinError),
    IO(std::io::Error),
    UnknownDaemon,
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

impl DaemonHandle {
    async fn future(self) -> Result<(), DaemonError> {
        match self {
            DaemonHandle::Process(child) => {
                Ok(child.await.map(|_| ())?)
            }
            DaemonHandle::Task(task) => {
                Ok(task.await?)
            }
        }
    }
}

#[async_trait]
impl TryService for Runtime {
    type ErrorType = RuntimeError;

    async fn try_run_loop(mut self) -> Result<!, RuntimeError> {
        let mut handlers = vec![];

        // TODO: return DaemonError
        handlers.push(self.daemon("stashd").unwrap());

        self.config
            .contracts
            .iter()
            .try_for_each(|contract_name| -> Result<(), RuntimeError> {
                // TODO: return DaemonError
                handlers.push(self.daemon(contract_name.daemon_name()).unwrap());
                Ok(())
            })?;

        join_all(handlers.into_iter().map(|d| d.future()))
            .await
            .into_iter()
            .try_for_each(|res| -> Result<(), RuntimeError> {
                // TODO: check `res` for errors, change return type to DaemonError
                Ok(())
            })?;

        unreachable!()
    }
}
