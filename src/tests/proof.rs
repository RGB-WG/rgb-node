use bitcoin::OutPoint;
use output_entry::OutputEntry;
use bitcoin::util::hash::Sha256dHash;
use proof::Proof;
use contract::Contract;
use traits::Verify;
use bitcoin::network::constants::Network;
use tests::rand::Rng;


fn make_txid() -> Sha256dHash {
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    Sha256dHash::from_hex(&hex::encode(random_bytes)).unwrap()
}

fn mock_contract(initial_owner_txid: Sha256dHash) -> Contract {
    Contract {
        title: "Fake contract".to_string(),
        issuance_utxo: OutPoint::default(),
        initial_owner_utxo: OutPoint {txid: initial_owner_txid, vout: 42},
        network: Network::Testnet,
        total_supply: 7
    }
}

fn mock_root_proof(contract: Option<Box<Contract>>, initial_owner_txid: Sha256dHash) -> Proof {
    Proof {
        bind_to: vec![OutPoint {txid: initial_owner_txid, vout: 42}],
        input: Vec::new(),
        output: Vec::new(),
        contract
    }
}

fn mock_proof(bind_to: Vec<OutPoint>, input: Vec<Proof>, output: Vec<OutputEntry>) -> Proof {
    Proof {bind_to, input, output, contract: None}
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
