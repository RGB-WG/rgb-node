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
use tokio::process;

use lnpbp::TryService;

use super::Config;
use crate::error::{BootstrapError, RuntimeError};

pub struct Runtime {
    config: Config,
}

impl Runtime {
    pub async fn init(config: Config) -> Result<Self, BootstrapError> {
        Ok(Self { config })
    }

    fn daemon(&self, bin: &str) -> process::Command {
        let mut daemon = self.config.bin_dir.clone();
        daemon.push(bin);
        let mut cmd = process::Command::new(daemon);
        cmd.args(&[
            "-v",
            "-v",
            "-v",
            "-v",
            "--data-dir",
            self.config.data_dir.to_str().expect("Binary path is wrong"),
        ]);
        cmd
    }
}

#[async_trait]
impl TryService for Runtime {
    type ErrorType = RuntimeError;

    async fn try_run_loop(mut self) -> Result<!, RuntimeError> {
        let mut handlers = vec![];

        handlers.push(self.daemon("stashd").spawn()?);

        self.config
            .contracts
            .iter()
            .try_for_each(|contract_name| -> Result<(), RuntimeError> {
                handlers.push(self.daemon(contract_name.daemon_name()).spawn()?);
                Ok(())
            })?;

        join_all(handlers)
            .await
            .into_iter()
            .try_for_each(|res| -> Result<(), RuntimeError> {
                res?;
                Ok(())
            })?;

        unreachable!()
    }
}
