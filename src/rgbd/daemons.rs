// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::process::Command;

use microservices::error::BootstrapError;
use microservices::{DaemonHandle, Launcher, LauncherError};

use super::Runtime;
use crate::{containerd, Config, LaunchError};

/// Daemons that can be launched by lnpd
#[derive(Clone, Eq, PartialEq, Debug, Display)]
pub enum Daemon {
    #[display("containerd")]
    Containerd,
}

impl Launcher for Daemon {
    type RunError = BootstrapError<LaunchError>;
    type Config = Config;

    fn bin_name(&self) -> &'static str {
        match self {
            Daemon::Containerd => "containerd",
        }
    }

    fn cmd_args(&self, cmd: &mut Command) -> Result<(), LauncherError<Self>> {
        cmd.args(std::env::args().skip(1));

        Ok(())
    }

    fn run_impl(self, config: Config) -> Result<(), BootstrapError<LaunchError>> {
        match self {
            Daemon::Containerd => containerd::run(config),
        }
    }
}

impl Runtime {
    pub(super) fn launch_daemon(
        &self,
        daemon: Daemon,
        config: Config,
    ) -> Result<DaemonHandle<Daemon>, LauncherError<Daemon>> {
        if self.config.threaded {
            daemon.thread_daemon(config)
        } else {
            daemon.exec_daemon()
        }
    }
}
