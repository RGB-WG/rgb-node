extern crate jsonrpc;
extern crate strason;

use clap::ArgMatches;
use database::Database;
use std::fs;
use std::path::{Path, PathBuf};

pub mod issueasset;
pub mod listunspent;
pub mod sendtoaddress;
pub mod getnewaddress;
pub mod sync;
pub mod burn;
pub mod deserialize;

pub trait RGBSubCommand<'a> {
    fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut jsonrpc::client::Client) -> Result<(), jsonrpc::Error>;
}

#[derive(Debug)]
pub struct Config {
    pub basedir: Box<PathBuf>,

    pub rpcconnect: String,
    pub rpcport: u16,
    pub rpcuser: String,
    pub rpcpassword: String,

    pub rgb_server: String,
}

impl Config {
    pub fn load_from(path: &Path) -> Config {
        let path = Path::new(path).join("rgb.conf");

        let json = match fs::File::open(&path) {
            Ok(file) => strason::Json::from_reader(file).unwrap(),
            Err(_) => strason::Json::from_str("{}").unwrap()
        };

        Config {
            basedir: Box::new(path.to_owned()),

            rpcconnect: json.get("rpcconnect").unwrap_or(&strason::Json::from("127.0.0.1")).string().unwrap().to_string(),
            rpcport: json.get("rpcport").unwrap_or(&strason::Json::from(18332)).num().unwrap().parse().unwrap(),
            rpcuser: json.get("rpcuser").unwrap_or(&strason::Json::from("satoshi")).string().unwrap().to_string(),
            rpcpassword: json.get("rpcpassword").unwrap_or(&strason::Json::from("nakamoto")).string().unwrap().to_string(),

            rgb_server: json.get("rgb_server").unwrap_or(&strason::Json::from("internal-rgb-bifrost.herokuapp.com")).string().unwrap().to_string(),
        }
    }
}