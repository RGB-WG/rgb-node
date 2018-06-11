extern crate bitcoin;
extern crate rgb;

use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::transaction::TxOutRef;
use bitcoin::network::constants::Network;
use bitcoin::network::encodable::ConsensusEncodable;
use bitcoin::util::hash::Sha256dHash;
use bitcoin::util::uint::Uint256;
use rgb::entities::contracts::emission_contract::EmissionContractData;
use rgb::entities::proofs::dummy_proof::DummyProofData;
use rgb::entities::proofs::emission_contract_proof::EmissionContractProofData;
use rgb::entities::proofs::transfer_proof::TransferProofData;
use rgb::entities::rgb_output::RgbOutPoint;
use rgb::entities::rgb_output::RgbOutput;
use rgb::entities::traits::Verify;
use rgb::entities::traits::ContainsSignatures;
use std::collections::HashMap;
use rgb::entities::proofs::Proof;
use rgb::util::txs::BuildFromTx;

fn hex_to_bytes(hex: String) -> Vec<u8> {
    // Make vector of bytes from octets
    let mut bytes = Vec::new();
    for i in 0..(hex.len() / 2) {
        let res = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16);
        match res {
            Ok(v) => bytes.push(v),
            Err(e) => println!("Problem with hex: {}", e),
        };
    };

    bytes
}

fn bytes_to_hex(bytes: &Vec<u8>) -> String {
    bytes
        .iter()
        .map(|byte: &u8| -> String {
            format!("{:02x}", byte)
        })
        .fold(String::new(), |res, byte: String| {
            res + &byte
        })
}

fn main() {
    // 8 input, 8 output (:5)
    let tx_0: Result<Transaction, bitcoin::util::Error> = bitcoin::network::serialize::deserialize(&mut hex_to_bytes(String::from("010000000001087fba6d1f17b827b580e90834cacaae15e82fbc5c3d7e55a678a87b4df250743b0000000023220020f91406091c3c4c72108fd203797da9331483ddbd45e51ad3383eee9dada48e3a00000000c607f54e28229ccd95bd71bc55989bdf9412d041d3e6b05e2270880beaecce1900000000232200209c0b1b29187a327cb3036a0334f73244c2ae7129665dfd1c3abb5b77440e3f1d00000000ec5a99944f772abaa2daf8224cf2f28f20cdf27cd34210693bac30d4a48396aa660a000023220020f4dcbea4d5f304da4e3d21da84201cd32a7e4d5e4fc5eaf02009e115f623c94200000000f6d6e03cbb082ddeff0317a8001c2d1560197fac1553824538adb1f41eff224a0000000023220020b370e4364b1ec6e9f110cc6752ea1512770abbf869b92f8be1c4dd510b6a39590000000072c61d1662f6c7e95d5d3b61c3bd5ff4d3af332df66c37c146484a0fd02adb3c00000000232200202635700da7e27574acf9f48fc878e8fbd793ddf5d2477d96b21f2b8014d3750500000000dfa9eb5bd3fdd53df65cbd5de0c5f62dbfa8ad6cb33fa57a4dae34ccc0246438c6060000db00483045022100c57543dc0071a29e84601032cefeb06e12c9cd36bd2eb7eceaaf3c4de337aec6022028078d8954e39cdeb47f368420e6188603f73ee43c9810f30c3d1eb5d5d780ae01483045022100ba96deec863a8df1cc04b58723f80b778aa6423734977fd233dbf965964f88ec02202aac8c114b4ea1b132f7ddf0fa622679eb6f8ac8475cb74e95ca962023030fd30147522102150a5f9ccbbd3b4bd5f2c6e511547c9c427223e3647f770366da34da7055c25c210292302d3fa1e23696caf5136adfac78b3dfc710d81f26557f5970390938e5204852ae00000000ec5a99944f772abaa2daf8224cf2f28f20cdf27cd34210693bac30d4a48396aa6e0b000023220020de884e8cc998b11d9f02d6e4eff24f4327cfe1c478e8ee29045ca37b9f0c4fc9000000002d39f0301f0d3c80131d858f77a4fb6a4a2c4e92791d43d8c568fe3ab01a5df60000000023220020d2baab246de40987f8dbfc35569a70a05282d41fab254699d3859a26af01e59f00000000083ecbdf00000000001976a91493ae94b7dd4168efd6de87fc9f20f5367051eb7088aca7369300000000001976a9141b75278a7a3cf3e5817ed15c328b97e1a7d7025e88ac749a1500000000001976a914cd5105757f4a6c54396a14c533146a73e7a99f6b88aca0bb0d00000000001976a914c8c2a90639143bfe84b52d8448a584f35a7b2f0b88ac80234300000000001976a914d103104cd0f0ae070c2e077b12e12683dd48af0f88accd524400000000001976a914e3318e4f36eeb551679c773f302b72c1285bb2d088ac62641300000000001976a9149b6ef03130bcab11f8a60dc68ae91f466779870488ace31b31000000000017a9148aef8ac462587a4061727612cc28b9407b3090c8870400483045022100ac2087a92124bd971ba6974629206fad28ada2cfc028c448eb7315a6842a9daf02204fb94fdd7db05775239c2260a3d01acf5a9a5d4385af115359917a5c76ddc8ed01473044022066304530d4b75846ff8cb4b77fa16741cf43ca93dff615f5985786b2863eadc402203182d6428fa2e942374703e80b7543c42b9db9445ebabddd42788e10515248210147522102707c54ec560ac2802f2948ebe3357a00a33757e30ca9177e6502d89307b818a3210320d8c2d7a639385fdb7bf4f160745bcc0fff3f9ee3afc054c21b62ed4773328c52ae04004830450221008cf2cb761c37e79c07d3efa87408c77829020547b9c45c419e8486326fb46a51022050fbad18e8f2b6f58c83b8fbb7bee2198470ace1d965d6b6def8379f3058b73e0147304402207f697287e38ac20dd7197e9b79cb9d6570a5b50fdb3fd5bd821b7cd58c56f19f022047401267368ea6be0eb88506acc940c79fd973891e72f0e3c95540111986b9dd01475221038f35245fd0afb932486a3c9fe441ac8f69aeece1549f3690bc98132930b170eb210231eddaacc7648dc25adccdbe530e2db6e93c01d2d625c51564de9a6fd999f0be52ae0400483045022100c5371dd0dc371ed7b394c877821f0bd32d70dc2f06df39e9465146c6b19e3e0402205fa260d081540e8ab4281e30fc783fcc963e88f393b1dbd080e42add23ac697f01483045022100f0a8095d51b4e400a60165eef4102445238e0d7a282e4f7ede9f203d2143b18e02205505b6f994aa065387990a3dd0a0a55c74eadf8327d92f3a3b4bdc6d57344bb90147522102cc7c7560fa5c80ae51725c1b77bef8de4891ad22814a2b3f16f5a970ca54d1be2103d2f9c71bb59823562c027f8ccb45fd8ba9ef7222eefa2da0c294f290ce5c3d7252ae0400483045022100d0ad7ce6534d9f3e457b5c3b40c33c4624a0c5cf9b9d26030059845ae1c0988302202984138ca0df2da09188292b3011f5c44192990c38927763ca1fbff6ebd85926014730440220268a2402168932f78c3f87f323668322e759256ad07c5a78b2f22440f4db53c6022069a89eeb95d71b5b1945a94c6b3f7da9a08db8ad68a9445aa5cac67192ffb6740147522103b96c1a4f33228607d18b170d527e7f2eed5a5dcdc81fbb7a9bbf0893446f130321028da878423999c52a60dfce71d228d1ada37634ca59d6d457a715545521877a2d52ae04004730440220070205df7ac851fd0765d379c728de7eadc3414f6f890c4145a6e7ee649bd15b02207e9e975efb5b64d8146ca16374fe61ac8960be5abb2715dd3b7fea8a4b69def5014730440220159f667362af02981c52b8c48d7601341c8113c3238c649fd635768129a517a602202c1da3cd788852b55ffb035562461e431be505f190eb29c219e6bfb65be41c6001475221026c5c27d527134be2e6200444c7011682a75d01dbd0adf10c0a28a46053a5144b21039aaa9a535b0d8e0fd27a5efe3b66c7043fbdbef1184d812320ede1a3d683eab952ae000400483045022100fa5270a19b8c21529458b7b488e53674bcf5243c0f4c98ad425740f643f1954202207a0bfcdbccf523383119bc5454e9a0a4a7100196e308d855ca95a6154ccddce701483045022100ab7299fc26cdce29a3a7e945bcd7d48f46b411c9d522290c2915778b418094da02206b2685ff7925bd61bb6bbbfd96fa6fb049be27cdcb564a70fad5f6669ce402850147522102eb6843c7e093d2246f0be21d23b76b3156770767706092b48bf5f92516798bce2102f92f1f6c53897d64a1be863b2bab6a61bc03a4c361c314e4255586caf3f0e33e52ae040047304402201669f96c5747d1b40c2fb6c3433e669c00c561b9f310a6767018ba58e0d0d8b702201e30bfc0d1236291116dc4ca5703b08c6bbd9f4f27c0c6a0f44cd5a276d5aad401473044022070ffb9524e2c8718f3665a4c60d5ae851737371e2d4e0c7531f0e481234f6b76022044c5fc5061dc409d6a261e4a7bb6cef63feb1e59a0a0337a804fb99b4decd4540147522103acbabba3c5951da1cc6ded99ad9067a8d3d304ee96d5fe4fabcff5d94a5e4de321032e8df58f5aead59372151cde38f4f8633175e4cd7cd971ff5ae0a49ae2b7ce5852ae00000000")));
    let tx_0 = tx_0.unwrap();

    // 1 input, 2 output (:0)
    let tx_1: Result<Transaction, bitcoin::util::Error> = bitcoin::network::serialize::deserialize(&mut hex_to_bytes(String::from("0200000001e00c84af074901193025376eedcdb1f61d0f2e2d8c61f2659326b3b5592afd9c050000006b483045022100edd3a3718e12901903c29f3a27005159a4fdb6d3780253f20e956aa418a8e89702205da0633d54159a9f17f78ebc6cd49fd4f6c3242795c65b7345e03ba63f33b8590121020f330844e6c8bd7e55c941612898d6bd25759282c310b58a490c6f87efb4bdc1feffffff02061035000000000017a914bcec441c57d79063c64312a0cb244f51b7be1aae87e7340d00000000001976a914b5e120d618b1ed577417de6f922f973078e0177988acfbc60700")));
    let tx_1 = tx_1.unwrap();

    // 2 input, 2 output
    let tx_2: Result<Transaction, bitcoin::util::Error> = bitcoin::network::serialize::deserialize(&mut hex_to_bytes(String::from("02000000028f6c39bfe86804b93c8a6971c3392bf2b8e12e0a2d52b7a74b169e3fc0a6aeea010000006b483045022100a285ccbb28a7bfb306ee8f60087d1ad18925db4c212bbbf5ef070723c745563b02205c171a6ab00df7015dcafd5f1e91a6ad061f4fdfea72b6ed45abd8d83c4d0a3d0121027c554b0ac0a025682fb2610c42f9e3cc52261640e188d594a84889a097ef66c1feffffff7fc8d11fd812b0f24db3db9747078c51d904ca580987a9f6f97e7efea26fbc91670000006b483045022100a3bb2e170a64f161305e6e47b12e65ce451da1eff06fdb749a5a71faf93d18b302202d02d124919c4e7ad7ba78c54caae8fff9fd68ca968a09cf4b521da08277defa012102177260794b61d9474fdd2d8115f4c7c59eddac6021e311f36ba9b79da35cbc44feffffff029e1c0e00000000001976a9142fb7bd0a693e6ab3fe643d4debd9921c9d47bd6888ac12310000000000001976a9144e6d575c5e4b6b075c5cc3b854192320922b05d188ac57060800")));
    let tx_2 = tx_2.unwrap();

    let s_1 = hex_to_bytes(String::from("AABBCCDDEEFF"));
    let r_1 = hex_to_bytes(String::from("112233445566"));

    let issuance_utxo = TxOutRef::from_tx(&tx_0, 5).unwrap();
    let first_proof_utxo = TxOutRef::from_tx(&tx_1, 0).unwrap();
    let transfer_proof_utxo = TxOutRef::from_tx(&tx_2, 0).unwrap();

    let mut txmap = HashMap::new();

    let emission_contract = EmissionContractData::new(
        String::from("Test Contract"),
        String::from("This is a test"),
        String::from("http://test.com"),
        issuance_utxo.clone(),
        Network::Bitcoin,
        1000,
        10,
        10,
        RgbOutPoint::NewUTXO(0),
    );
    let token_id = emission_contract.token_id();

    println!("{:?}", token_id.be_hex_string()); // reversed

    txmap.insert(&issuance_utxo, tx_1.clone());
    txmap.insert(&first_proof_utxo, tx_2.clone());
    txmap.insert(&transfer_proof_utxo, tx_0.clone());

    println!("{:?}", emission_contract.verify(&txmap));

    let rgb_out_0: RgbOutput = RgbOutput::new(500, token_id, RgbOutPoint::new_utxo(transfer_proof_utxo.clone()));
    let rgb_out_1: RgbOutput = RgbOutput::new(500, token_id, RgbOutPoint::new_index(1));
    let mut root_proof = EmissionContractProofData::new(TxOutRef::from_tx(&tx_1, 0).unwrap(), &[], &[rgb_out_0, rgb_out_1], emission_contract);

    root_proof.header.push_signature(s_1, r_1);

    println!("{:?} {:?}", root_proof.verify(&txmap), root_proof);

    let rgb_out_2: RgbOutput = RgbOutput::new(500, token_id, RgbOutPoint::new_index(0));
    let next_proof = TransferProofData::new(TxOutRef::from_tx(&tx_2, 0).unwrap(), &[root_proof], &[rgb_out_2]);

    println!("{:?} {:?}", next_proof.verify(&txmap), next_proof);

    let mut encoded: Vec<u8> = Vec::new();
    let mut enc = bitcoin::network::serialize::RawEncoder::new(encoded);

    next_proof.consensus_encode(&mut enc);
    let enc_str = bytes_to_hex(&enc.into_inner());
    println!("{:?}", enc_str);

    let decoded: Proof = bitcoin::network::serialize::deserialize(&mut hex_to_bytes(String::from(enc_str))).unwrap();
    println!("{:#?}", decoded);
}