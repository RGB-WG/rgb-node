use bitcoin::Address;
use bitcoin::network::constants::Network;
use bitcoin::network::serialize::BitcoinHash;
use bitcoin::network::serialize::serialize;
use bitcoin::OutPoint;
use bitcoin::Script;
use bitcoin::util::hash::Sha256dHash;
use contract::Contract;
use proof::OutputEntry;
use proof::Proof;
use utils::bytes_to_hex;

fn create_demo_contract() -> Contract {
    Contract {
        title: String::from("Test Contract"),
        issuance_utxo: OutPoint::default(),
        initial_owner_utxo: OutPoint::default(),
        burn_address: Address::p2sh(&Script::new(), Network::Testnet),
        network: Network::Testnet,
        total_supply: 100,
    }
}

#[test]
fn serialize_root_proof() {
    let contract = create_demo_contract();

    // create an invalid root proof (no outputs)
    let root_proof = Proof::new(vec![OutPoint::default()], vec![], vec![], Some(&contract));

    let serialized = bytes_to_hex(&serialize(&root_proof).unwrap());
    let expected = String::from("010000000000000000000000000000000000000000000000000000000000000000ffffffff0000010d5465737420436f6e74726163740000000000000000000000000000000000000000000000000000000000000000ffffffff0000000000000000000000000000000000000000000000000000000000000000ffffffff23324e39684c776b5371723163505141507862724756556a78796a4431314732653168650b11090764000000");

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_normal_proof() {
    // create an invalid root proof (no outputs)
    let root_proof = Proof::new(vec![OutPoint::default()], vec![], vec![], None);

    let serialized = bytes_to_hex(&serialize(&root_proof).unwrap());
    let expected = String::from("010000000000000000000000000000000000000000000000000000000000000000ffffffff000000");

    assert_eq!(serialized, expected);
}

#[test]
fn hash_root_proof() {
    let contract = create_demo_contract();

    // create an invalid root proof (no outputs)
    let root_proof = Proof::new(vec![OutPoint::default()], vec![], vec![], Some(&contract));

    let hash = root_proof.bitcoin_hash();
    let expected = Sha256dHash::from_hex("9a538906e6466ebd2617d321f71bc94e56056ce213d366773699e28158e00614").unwrap();

    assert_eq!(hash, expected);
}

#[test]
fn proof_without_bind_to_fails() { // TODO
    assert_eq!(0, 0);
}

#[test]
fn check_fields_hashed_proof() {
    let contract = create_demo_contract();

    let out = OutputEntry::new(Sha256dHash::from_hex("9a538906e6466ebd2617d321f71bc94e56056ce213d366773699e28158e00614").unwrap(), 1000, 5);

    // create an invalid root proof (no outputs)
    let root_proof_original = Proof::new(vec![OutPoint::default()], vec![], vec![out.clone()], Some(&contract));
    let hash_original = root_proof_original.bitcoin_hash();

    // create an equivalent proof
    let root_proof_new = Proof::new(vec![OutPoint::default(), OutPoint::default()], vec![root_proof_original], vec![out], None);
    let hash_new = root_proof_new.bitcoin_hash();

    assert_eq!(hash_original, hash_new);
}

#[test]
fn check_fields_hashed_proof_ne() {
    let contract = create_demo_contract();

    let out = OutputEntry::new(Sha256dHash::from_hex("9a538906e6466ebd2617d321f71bc94e56056ce213d366773699e28158e00614").unwrap(), 1000, 5);

    // create an invalid root proof (no outputs)
    let root_proof_original = Proof::new(vec![OutPoint::default()], vec![], vec![out.clone()], Some(&contract));
    let hash_original = root_proof_original.bitcoin_hash();

    // create an equivalent proof
    let root_proof_new = Proof::new(vec![OutPoint::default()], vec![], vec![out.clone(), out.clone()], Some(&contract));
    let hash_new = root_proof_new.bitcoin_hash();

    assert_ne!(hash_original, hash_new);
}