use bitcoin::network::serialize::BitcoinHash;
use bitcoin::network::serialize::deserialize;
use bitcoin::OutPoint;
use bitcoin::util::hash::Sha256dHash;
use regex::Regex;
use rgb::proof::Proof;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Database {
    basedir: Box<PathBuf>
}

impl Database {
    pub fn new(basedir: &Path) -> Database {
        let db = Database {
            basedir: Box::new(basedir.to_owned())
        };

        db.init();

        db
    }

    fn init(&self) {
        if !self.basedir.as_path().exists() {
            fs::create_dir(self.basedir.as_path());
        }
    }

    pub fn list_local_proofs(&self) -> HashMap<Sha256dHash, Vec<Proof>> {
        // Regex to match <txid>:<vout> folders
        let outpoint_folder_regex = Regex::new(r"(?i)^[0-9a-f]{64}:[0-9]+$").unwrap();

        let mut ans = HashMap::new();

        for outpoint_dir in self.basedir.as_path().read_dir().expect("read_dir call failed") {
            if let Ok(outpoint_dir) = outpoint_dir {
                let file_name = outpoint_dir.file_name();
                let folder_name = file_name.to_str();

                if outpoint_folder_regex.is_match(folder_name.unwrap()) {
                    let txid = Sha256dHash::from_hex(folder_name.unwrap().split(":").next().unwrap()).unwrap();

                    let mut proofs = Vec::new();

                    for entry in outpoint_dir.path().read_dir().expect("read_dir call failed") {
                        if let Ok(entry) = entry {
                            let mut file = fs::File::open(entry.path()).unwrap();
                            let mut buffer: Vec<u8> = Vec::new();

                            file.read_to_end(&mut buffer);

                            let decoded: Proof = deserialize(&mut buffer).unwrap();
                            proofs.push(decoded);
                        }
                    }

                    ans.insert(txid, proofs);
                }
            }
        }

        ans
    }

    pub fn get_proofs_for(&self, outpoint: &OutPoint) -> Vec<Proof> {
        let mut ans = Vec::new();

        let outpoint_str = outpoint.txid.be_hex_string() + ":" + outpoint.vout.to_string().as_str();

        let mut proof_path = self.basedir.clone();
        proof_path.push(outpoint_str);

        if !proof_path.as_path().exists() {
            return ans;
        }

        for entry in proof_path.as_path().read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                let mut file = fs::File::open(entry.path()).unwrap();
                let mut buffer: Vec<u8> = Vec::new();

                file.read_to_end(&mut buffer);

                let decoded: Proof = deserialize(&mut buffer).unwrap();
                ans.push(decoded);
            }
        }

        ans
    }

    pub fn save_proof(&self, proof: &Proof, txid: &Sha256dHash) {
        for out in &proof.output {
            let outpoint_str = match out.get_vout() {
                Some(vout) => txid.be_hex_string() + ":" + vout.to_string().as_str(),
                None => txid.be_hex_string() + ":BURN"
            };

            let mut proof_path = self.basedir.clone();
            proof_path.push(outpoint_str);
            proof_path.push(proof.bitcoin_hash().be_hex_string());

            if !proof_path.as_path().parent().unwrap().exists() {
                fs::create_dir_all(&proof_path.as_path().parent().unwrap());
            }

            use bitcoin::network::serialize::RawEncoder;
            use bitcoin::network::encodable::ConsensusEncodable;

            let mut encoded: Vec<u8> = Vec::new();
            let mut enc = RawEncoder::new(encoded);
            proof.consensus_encode(&mut enc);

            let mut doc = fs::File::create(proof_path.as_path()).unwrap();
            doc.write(&enc.into_inner());
        }
    }
}