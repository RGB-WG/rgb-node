pub fn hex_to_bytes(hex: String) -> Vec<u8> {
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

pub fn bytes_to_hex(bytes: &Vec<u8>) -> String {
    bytes
        .iter()
        .map(|byte: &u8| -> String {
            format!("{:02x}", byte)
        })
        .fold(String::new(), |res, byte: String| {
            res + &byte
        })
}