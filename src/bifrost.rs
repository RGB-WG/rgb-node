use bitcoin::OutPoint;
use bitcoin::util::hash::Sha256dHash;
use hyper::Client;
use hyper::Error;
use hyper::header::{ContentType, Headers};
use hyper::mime::{Mime, SubLevel, TopLevel};
use rgb::proof::Proof;
use std::io;
use std::io::Read;

pub fn upload_proofs(server: &String, proof: &Proof, txid: &Sha256dHash) -> Result<(), Error> {
    for out in &proof.output {
        let outpoint_str = match out.get_vout() {
            Some(vout) => txid.be_hex_string() + ":" + vout.to_string().as_str(),
            None => txid.be_hex_string() + ":BURN"
        };
        let url = format!("http://{}/{}", server, outpoint_str);

        let client = Client::new();
        let mut headers = Headers::new();
        headers.set(ContentType("application/octet-stream".parse().unwrap()));

        use bitcoin::network::serialize::RawEncoder;
        use bitcoin::network::encodable::ConsensusEncodable;

        let mut encoded: Vec<u8> = Vec::new();
        let mut enc = RawEncoder::new(encoded);
        proof.consensus_encode(&mut enc);

        let request_raw = &enc.into_inner();

        // Copied from rust-jsonrpc (@apoelstra)
        let retry_headers = headers.clone();
        let hyper_request = client.post(&url).headers(headers).body(&request_raw[..]);
        let mut stream = match hyper_request.send() { // TODO: error handling
            Ok(s) => s,
            // Hyper maintains a pool of TCP connections to its various clients,
            // and when one drops it cannot tell until it tries sending. In this
            // case the appropriate thing is to re-send, which will cause hyper
            // to open a new connection. Jonathan Reem explained this to me on
            // IRC, citing vague technical reasons that the library itself cannot
            // do the retry transparently.
            Err(Error::Io(e)) => {
                if e.kind() == io::ErrorKind::ConnectionAborted {
                    client.post(&url)
                        .headers(retry_headers)
                        .body(&request_raw[..])
                        .send()?
                } else {
                    return Err(Error::Io(e));
                }
            }
            Err(e) => { return Err(e); }
        };
    }

    Ok(())
}

pub fn get_proofs_for(server: &String, outpoint: &OutPoint) -> Result<Vec<Proof>, Error> {
    use bitcoin::network::serialize::deserialize;

    let outpoint_str = outpoint.txid.be_hex_string() + ":" + outpoint.vout.to_string().as_str();
    let url = format!("http://{}/{}", server, outpoint_str);

    let client = Client::new();
    let hyper_request = client.get(&url);
    let mut stream = match hyper_request.send() { // TODO: error handling
        Ok(s) => s,
        Err(Error::Io(e)) => { // retry
            if e.kind() == io::ErrorKind::ConnectionAborted {
                client.get(&url).send()?
            } else {
                return Err(Error::Io(e));
            }
        }
        Err(e) => { return Err(e); }
    };

    let mut buffer: Vec<u8> = Vec::new();
    stream.read_to_end(&mut buffer);

    let decoded: Vec<Proof> = deserialize(&mut buffer).unwrap();

    Ok(decoded)
}