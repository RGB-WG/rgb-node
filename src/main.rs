extern crate bitcoin;
extern crate clap;
extern crate core;
extern crate hyper;
extern crate jsonrpc;
extern crate regex;
extern crate rgb;

use clap::{App, Arg, SubCommand};
use kaleidoscope::burn::Burn;
use kaleidoscope::getnewaddress::GetNewAddress;
use kaleidoscope::issueasset::IssueAsset;
use kaleidoscope::listunspent::ListUnspent;
use kaleidoscope::RGBSubCommand;
use kaleidoscope::sendtoaddress::SendToAddress;
use kaleidoscope::sync::Sync;
use std::env::home_dir;
use std::path::Path;

pub mod kaleidoscope;
pub mod database;
pub mod chain;
pub mod bifrost;

fn main() {
    const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
    const AUTHORS: Option<&'static str> = option_env!("CARGO_PKG_AUTHORS");

    // TODO: add --dry-run
    let matches = App::new("RGB - Kaleidoscope Client")
        .version(VERSION.unwrap_or("<unknown>"))
        .author(AUTHORS.unwrap_or("<unknown>"))
        .about("<TODO: write the about section>") // TODO
        .arg(Arg::with_name("datadir")
            .short("d")
            .long("datadir")
            .value_name("DIRECTORY")
            .help("Sets a data directory")
            .takes_value(true))
        .subcommand(SubCommand::with_name("issueasset")
            .about("Issue a new asset")
            .arg(Arg::with_name("title")
                .takes_value(true)
                .long("title")
                .value_name("TITLE")
                .required(true)
                .help("Set the title"))
            .arg(Arg::with_name("total_supply")
                .takes_value(true)
                .long("supply")
                .value_name("VALUE")
                .required(true)
                .help("Set the total supply"))
            .arg(Arg::with_name("issuance_utxo")
                .takes_value(true)
                .long("issuance_utxo")
                .value_name("UTXO")
                .help("Set the UTXO which will commit to the contract"))
            .arg(Arg::with_name("initial_owner_utxo")
                .takes_value(true)
                .long("initial_owner_utxo")
                .value_name("UTXO")
                .help("Set the UTXO which will receive all the tokens initially"))
            .arg(Arg::with_name("network")
                .takes_value(true)
                .long("network")
                .value_name("NETWORK")
                .help("Set the network")
                .default_value("testnet")))
        .subcommand(SubCommand::with_name("listunspent")
            .about("List the unspent Bitcoin outputs and RGB proofs"))
        .subcommand(SubCommand::with_name("getnewaddress")
            .about("Generate a new RGB address")
            .arg(Arg::with_name("server")
                .takes_value(true)
                .long("server")
                .value_name("SERVER[:PORT]")
                .help("Overrides the default server")))
        .subcommand(SubCommand::with_name("sync")
            .about("Synchronize with the server")
            .arg(Arg::with_name("server")
                .takes_value(true)
                .long("server")
                .value_name("SERVER[:PORT]")
                .help("Overrides the default server"))
            .arg(Arg::with_name("upload-all")
                .long("upload-all")
                .short("a")
                .help("Upload every proof instead of the ones you own"))
        )
        .subcommand(SubCommand::with_name("sendtoaddress")
            .about("Send some RGB tokens to a specified address")
            .arg(Arg::with_name("address")
                .value_name("ADDRESS")
                .required(true)
                .help("Set the address"))
            .arg(Arg::with_name("asset_id")
                .value_name("ASSET_ID")
                .required(true)
                .help("Set the asset_id"))
            .arg(Arg::with_name("amount")
                .value_name("AMOUNT")
                .required(true)
                .help("Set the amount")))
        .subcommand(SubCommand::with_name("burn")
            .about("Burn some RGB tokens")
            .arg(Arg::with_name("asset_id")
                .value_name("ASSET_ID")
                .required(true)
                .help("Set the asset_id"))
            .arg(Arg::with_name("amount")
                .value_name("AMOUNT")
                .required(true)
                .help("Set the amount")))
        .get_matches();

    let default_rgb_dir = home_dir().unwrap().join(".rgb");
    let datadir = Path::new(matches.value_of("datadir").unwrap_or(default_rgb_dir.to_str().unwrap()));
    let datadirDataStr = [matches.value_of("datadir").unwrap_or(default_rgb_dir.to_str().unwrap()), "/data"].join("");
    let datadirData = Path::new(&datadirDataStr);
    let config = kaleidoscope::Config::load_from(datadir);
    let mut database = database::Database::new(&datadirData);
    let mut client = jsonrpc::client::Client::new("http://".to_owned() + &config.rpcconnect + &":".to_owned() + &config.rpcport.to_string(), Some(config.rpcuser.clone()), Some(config.rpcpassword.clone()));

    // ---------------------------

    if let Some(sub_matches) = matches.subcommand_matches("issueasset") {
        IssueAsset::run(&sub_matches, &config, &mut database, &mut client);
    } else if let Some(sub_matches) = matches.subcommand_matches("listunspent") {
        ListUnspent::run(&sub_matches, &config, &mut database, &mut client);
    } else if let Some(sub_matches) = matches.subcommand_matches("burn") {
        Burn::run(&sub_matches, &config, &mut database, &mut client);
    } else if let Some(sub_matches) = matches.subcommand_matches("sendtoaddress") {
        SendToAddress::run(&sub_matches, &config, &mut database, &mut client);
    } else if let Some(sub_matches) = matches.subcommand_matches("getnewaddress") {
        GetNewAddress::run(&sub_matches, &config, &mut database, &mut client);
    } else if let Some(sub_matches) = matches.subcommand_matches("sync") {
        Sync::run(&sub_matches, &config, &mut database, &mut client);
    } else {
        println!("{}", matches.usage());
    }
}