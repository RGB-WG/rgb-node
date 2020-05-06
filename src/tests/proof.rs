use super::rand::Rng;
use traits::Verify;
use bitcoin::network::constants::Network;
use bitcoin::network::serialize::BitcoinHash;
use bitcoin::network::serialize::deserialize;
use bitcoin::network::serialize::serialize_hex;
use bitcoin::OutPoint;
use bitcoin::util::hash::Sha256dHash;
use contract::Contract;
use output_entry::OutputEntry;
use proof::Proof;
use utils::hex_to_bytes;

fn make_txid() -> Sha256dHash {
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    Sha256dHash::from_hex(&hex::encode(random_bytes)).unwrap()
}

fn mock_contract(initial_owner_txid: Sha256dHash) -> Contract {
    Contract {
        version: 0,
        title: "Fake contract".to_string(),
        issuance_utxo: OutPoint::default(),
        initial_owner_utxo: OutPoint {txid: initial_owner_txid, vout: 42},
        tx_committing_to_this: None,
        network: Network::Testnet,
        total_supply: 7,
        original_commitment_pk: None
    }
}

fn mock_root_proof(contract: Option<Box<Contract>>, initial_owner_txid: Sha256dHash) -> Proof {
    Proof {
        version: 0,
        bind_to: vec![OutPoint {txid: initial_owner_txid, vout: 42}],
        input: Vec::new(),
        output: Vec::new(),
        tx_committing_to_this: None,
        contract,
        original_commitment_pk: None
    }
}

fn mock_proof(bind_to: Vec<OutPoint>, input: Vec<Proof>, output: Vec<OutputEntry>) -> Proof {
    Proof { version: 0, bind_to, input, output, tx_committing_to_this: None, contract: None, original_commitment_pk: None }
}

fn create_demo_contract() -> Contract {
    Contract {
        version: 0,
        title: String::from("Test Contract"),
        issuance_utxo: OutPoint::default(),
        initial_owner_utxo: OutPoint::default(),
        network: Network::Testnet,
        total_supply: 100,
        tx_committing_to_this: None,
        original_commitment_pk: None
    }
}

// TODO: test consensus rules

#[test]
fn serialize_output_entry() {
    let entry = OutputEntry::new(Sha256dHash::default(), 100, Some(255));

    let serialized = serialize_hex(&entry).unwrap();
    let expected = String::from("0000000000000000000000000000000000000000000000000000000000000000640000000000000001ff000000");

    assert_eq!(serialized, expected);
}

#[test]
fn deserialize_output_entry() {
    let serialized = String::from("0000000000000000000000000000000000000000000000000000000000000000640000000000000001ff000000");
    let entry: OutputEntry = deserialize(&hex_to_bytes(serialized)).unwrap();

    assert_eq!(entry.get_asset_id(), Sha256dHash::default());
    assert_eq!(entry.get_amount(), 100);
    assert_eq!(entry.get_vout(), Some(255));
}

#[test]
fn serialize_root_proof() {
    let contract = create_demo_contract();

    // create an invalid root proof (no outputs)
    let root_proof = Proof::new(vec![OutPoint::default()], vec![], vec![], Some(&contract), None);

    let serialized = serialize_hex(&root_proof).unwrap();
    let expected = String::from("0100010000000000000000000000000000000000000000000000000000000000000000ffffffff0000000100000d5465737420436f6e74726163740000000000000000000000000000000000000000000000000000000000000000ffffffff0000000000000000000000000000000000000000000000000000000000000000ffffffff0b1109076400000000000000000000");

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_normal_proof() {
    // create an invalid root proof (no outputs)
    let root_proof = Proof::new(vec![OutPoint::default()], vec![], vec![], None, None);

    let serialized = serialize_hex(&root_proof).unwrap();
    let expected = String::from("0100010000000000000000000000000000000000000000000000000000000000000000ffffffff0000000000");

    assert_eq!(serialized, expected);
}

#[test]
fn hash_root_proof() {
    let contract = create_demo_contract();

    // create an invalid root proof (no outputs)
    let root_proof = Proof::new(vec![OutPoint::default()], vec![], vec![], Some(&contract), None);

    let hash = root_proof.bitcoin_hash();
    let expected = Sha256dHash::from_hex("9a538906e6466ebd2617d321f71bc94e56056ce213d366773699e28158e00614").unwrap();

    assert_eq!(hash, expected);
}

#[test]
fn check_fields_hashed_proof() {
    let contract = create_demo_contract();

    let out = OutputEntry::new(Sha256dHash::from_hex("9a538906e6466ebd2617d321f71bc94e56056ce213d366773699e28158e00614").unwrap(), 1000, Some(5));

    // create an invalid root proof (no outputs)
    let root_proof_original = Proof::new(vec![OutPoint::default()], vec![], vec![out.clone()], Some(&contract), None);
    let hash_original = root_proof_original.bitcoin_hash();

    // create an equivalent proof
    let root_proof_new = Proof::new(vec![OutPoint::default(), OutPoint::default()], vec![root_proof_original], vec![out], None, None);
    let hash_new = root_proof_new.bitcoin_hash();

    assert_eq!(hash_original, hash_new);
}

#[test]
fn check_fields_hashed_proof_ne() {
    let contract = create_demo_contract();

    let out = OutputEntry::new(Sha256dHash::from_hex("9a538906e6466ebd2617d321f71bc94e56056ce213d366773699e28158e00614").unwrap(), 1000, Some(5));

    // create an invalid root proof (no outputs)
    let root_proof_original = Proof::new(vec![OutPoint::default()], vec![], vec![out.clone()], Some(&contract), None);
    let hash_original = root_proof_original.bitcoin_hash();

    // create an equivalent proof
    let root_proof_new = Proof::new(vec![OutPoint::default()], vec![], vec![out.clone(), out.clone()], Some(&contract), None);
    let hash_new = root_proof_new.bitcoin_hash();

    assert_ne!(hash_original, hash_new);
}

#[test]
fn get_needed_txs() {
    let initial_owner_txid = make_txid();
    let contract = mock_contract(initial_owner_txid);
    let root = mock_root_proof(Some(Box::new(contract)), initial_owner_txid);

    let needed_txs = root.get_needed_txs();
    // 1 tx: contract
    // 1 tx: root proof
    assert_eq!(needed_txs.len(), 2);

    // Transaction #2
    let root_outpoint_0 = OutPoint {
        txid: make_txid(),
        vout: 42
    };
    let proof_1 = mock_proof(vec![root_outpoint_0], vec![root.clone()], vec![]);
    assert_eq!(proof_1.get_needed_txs().len(), 3);

    // Transaction #3
    let root_outpoint_1 = OutPoint {
        txid: root_outpoint_0.txid,
        vout: 43
    };
    let proof_2 = mock_proof(vec![root_outpoint_1], vec![root.clone()], vec![]);
    assert_eq!(proof_2.get_needed_txs().len(), 3);

    // Transaction #4
    let outpoint_tx2 = OutPoint {
        txid: make_txid(),
        vout: 42
    };
    let outpoint_tx3 = OutPoint {
        txid: make_txid(),
        vout: 42
    };
    let bind_to_3 = vec![outpoint_tx2, outpoint_tx3];
    let input_3 = vec![proof_1, proof_2];
    let proof_3 = mock_proof(bind_to_3, input_3, vec![]);
    // The proof_3.get_needed_txs() vector has 2 duplicated entries
    // TODO: check this behavior
    assert_eq!(proof_3.get_needed_txs().len(), 8);
}
