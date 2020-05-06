use kaleidoscope::{Config, RGBSubCommand};
use clap::ArgMatches;
use database::Database;
use jsonrpc::client::Client;
use std::fs;
use rgb::proof::Proof;
use std::io::Read;

pub struct Deserialize {}

impl<'a> RGBSubCommand<'a> for Deserialize {
	fn run(matches: &'a ArgMatches<'a>, config: &Config, database: &mut Database, client: &mut Client) -> Result<(), jsonrpc::Error> {
		use bitcoin::network::serialize::deserialize;
		let path = matches.value_of("path").unwrap();
		println!("Deserializing proof at {}", path);
		let mut file = fs::File::open(path).unwrap();
		let mut buffer: Vec<u8> = Vec::new();

		file.read_to_end(&mut buffer);

		let decoded: Proof = deserialize(&mut buffer).unwrap();
		println!("Proof at {} deserialized succesfully", path);
		Ok(())
	}
}