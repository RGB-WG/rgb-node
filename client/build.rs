// RGB Node: sovereign smart contracts backend
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
// Written in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2020-2024 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2025 RGB Consortium, Switzerland. All rights reserved.
// Copyright (C) 2020-2025 Dr Maxim Orlovsky. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

#[macro_use]
extern crate clap;
#[macro_use]
extern crate amplify;

use std::fs;

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::*;

pub mod cli {
    include!("src/args.rs");
}

fn main() -> Result<(), configure_me_codegen::Error> {
    let outdir = "../shell";

    fs::create_dir_all(outdir).expect("failed to create shell dir");
    #[allow(clippy::single_element_loop)]
    for app in [cli::Args::command()].iter_mut() {
        let name = app.get_name().to_string();
        generate_to(Bash, app, &name, outdir)?;
        generate_to(PowerShell, app, &name, outdir)?;
        generate_to(Zsh, app, &name, outdir)?;
    }

    // configure_me_codegen::build_script_auto()
    Ok(())
}
