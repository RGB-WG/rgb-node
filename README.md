# RGB - Kaleidoscope

## Installation

1. Install Cargo: `curl -sSf https://static.rust-lang.org/rustup.sh | sh`
2. Build the project: `cargo build`

When the build is completed, the executable will be located at `./target/debug/kaleidoscope`.

For convenience, it can be useful to temporarily add the directory to your `PATH`, like so:

```
export PATH=$(readlink -f ./target/debug):$PATH
```

Make sure that you can now run the executable with:

```
kaleidoscope --version
```

## Configuration

RGB, like Bitcoin, has a "home" data directory, which is used to store the database of proofs and might contain a configuration file (`rgb.conf`).

By default, the data directory is `$HOME/.rgb`. This can be overridden by adding the `--datadir <NEWDIR>` (or `-d <NEWDIR>`) to each command.

If the directory does not exist, RGB will create it.

### Preparing `bitcoind`

`bitcoind` needs to be fully synced with the `txindex` enabled.

The node should also only use legacy addresses (`addresstype=legacy`).

### Example configuration file

The configuration file is located at `<datadir>/rgb.conf`.

If the file is missing, it will use as default values the ones shown here.

```json
{
    "rpcconnect": "127.0.0.1",
    "rpcport": 18332,
    "rpcuser": "satoshi",
    "rpcpassword": "nakamoto",

    "default_server": "internal-rgb-bifrost.herokuapp.com"
}
```

If any of the parameters is omitted, it will have as a default value the one corresponding one shown here.

The first four parameters are used to connect to `bitcoind`, while the last one is the server address which will be embedded into the invoices generated to request the upload of proofs.

## Running RGB

Most of the RGB commands keep syntax very similar to some known `bitcoin-cli` commands:

A list of accepted parameters and some other useful information is returned when `kaleidoscope help <command>` is run.

### `getnewaddress`

Returns and "RGB" address, which is actually a Bitcoin address with the indication of a server used to upload proofs.

Example:

```
$ kaleidoscope getnewaddress
mhmEj3thQZ4JBJ1ZiHDnNzj3bcixauQZ9Q@127.0.0.1
```

### `listunspent`

Returns the list of Bitcoin unspent outputs and the RGB proofs bound to them (if any).

Example:

```
$ kaleidoscope listunspent
+---------------------------------------------------------------------+
|  7ecd36b1f33b8313468425c62bc9c5c2b8f3970b08f1cd6c104da5095a975382:0 |
|    Amount:      1961210 SAT                                         |
+---------------------------------------------------------------------+
|                          ! NO RGB Proofs !                          |
+---------------------------------------------------------------------+

+---------------------------------------------------------------------+
|  fbaf1f4cf45dd4c6d3a285e768e5b413dc2cb3edf2b0fb698f6025609277ffee:0 |
|    Amount:       978455 SAT                                         |
+---------------------------------------------------------------------+
|  60a5ddd483eda2d45d5564e2ff1a62dda37384b8f982b7a0c0b9dcfbc760c415   | <-- asset_id
|    Amount:         1000                                             | <-- amount
+---------------------------------------------------------------------+

+---------------------------------------------------------------------+
|  4184d9279ecfb66347be3951820bbdedc01d7f74fca2a8651311f14e0a9f4fb2:0 |
|    Amount:         9100 SAT                                         |
+---------------------------------------------------------------------+
|                          ! NO RGB Proofs !                          |
+---------------------------------------------------------------------+
```

### `sendtoaddress`

Sends some tokens to an RGB address

Example:

```
$ kaleidoscope sendtoaddress mhmEj3thQZ4JBJ1ZiHDnNzj3bcixauQZ9Q@127.0.0.1 60a5ddd483eda2d45d5564e2ff1a62dda37384b8f982b7a0c0b9dcfbc760c415 400
Created a new TX with the following outputs:
         400 of 60a5ddd483eda2d45d5564e2ff1a62dda37384b8f982b7a0c0b9dcfbc760c415 to mhmEj3thQZ4JBJ1ZiHDnNzj3bcixauQZ9Q
         488227 SAT to mhmEj3thQZ4JBJ1ZiHDnNzj3bcixauQZ9Q
[CHANGE] 600 of 60a5ddd483eda2d45d5564e2ff1a62dda37384b8f982b7a0c0b9dcfbc760c415 to mieA9x3zbUNt83FmEwfwPVZigWy2UCno7F
[CHANGE] 488228 SAT to mieA9x3zbUNt83FmEwfwPVZigWy2UCno7F
TXID: 78e48fd199fb5b64c6f5321e7f7bd7d8d9d5a324ff98acb91b62e3c6b011bcb2
```

### `issueasset`

Issues a new asset.

This requires at least two UTXOs present in the `bitcoind` wallet **without** any RGB proof attached to them, which will be spent during the issuance process.

```
$ kaleidoscope issueasset --network regtest --title "My Awesome Asset" --supply 1000
Asset ID: a0ac6c10ba669b1e6c833e05c366c1f0c2535dbd5b71ecb52902a2604823409a
Spending the issuance_utxo OutPoint { txid: da036035876a60a0ae5e5e828868e00e3c2ebc12fa33682d32c1d8cf4a916b30, vout: 0 } in 9138089adbc5699d00100e8f7191936ae4332c46047d6d1a4fef3827f13af5a8
Spending the initial_owner_utxo OutPoint { txid: d641a6a9d8476159c6f58cba83ea8a349839738abea9a99d34bda1d98fb1fa80, vout: 0 } in 82eb4aab347e180ed45009c8c9ba094c4f38097911c80d01d53446cac21c4559
```

### `sync`

Synchronizes proofs with your server.

Downloaded (received) proofs are validated at this stage.

```
$ kaleidoscope sync
 --> Uploaded proof ef6acd09ce2dff2e7accf34b0a580d6428363851e0461cda87fcead65ce28cd7
```

### `burn`

Burns some assets by sending them to the public burn address specified in the contract.

This is equivalent to a `sendtoaddress <burn-address>`.

```
$ kaleidoscope burn 19fb2674bf892a2dfff0bb5694b4441b2707adf2ae6928948746c65f334fe450 100
Created a new TX with the following outputs:
                100 of 19fb2674bf892a2dfff0bb5694b4441b2707adf2ae6928948746c65f334fe450 to mtn51FF3zMkKKzCkK8iTX3N6uQRJhfm5PM
                2499997500 SAT to mtn51FF3zMkKKzCkK8iTX3N6uQRJhfm5PM
       [CHANGE] 900 of 19fb2674bf892a2dfff0bb5694b4441b2707adf2ae6928948746c65f334fe450 to mzBPC82VU3uvCbdGBJxmoQYAHSebqFvfdm
       [CHANGE] 2499997500 SAT to mzBPC82VU3uvCbdGBJxmoQYAHSebqFvfdm
```

## Appendix

### Example `bitcoin.conf`

```
# [core]
# Maintain a full transaction index, used by the getrawtransaction rpc call.
txindex=1

# [debug]
testnet=1

# [rpc]
# Accept command line and JSON-RPC commands.
rpcuser=satoshi
rpcpassword=nakamoto
server=1

debug=rpc
debug=mempool

daemon=1
addresstype=legacy
```