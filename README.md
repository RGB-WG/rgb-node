# RGB Node & SDK

[![TravisCI](https://api.travis-ci.com/LNP-BP/rgb-node.svg?branch=master)](https://api.travis-ci.com/LNP-BP/rgb-node)

This repository contains RGB node source code and SDK for wallet & server-side
development.

The node may run as a set of daemons (even in different docker containers);
a multi-threaded single process or as a set of managed threads within a
wallet app.

## Build

### Local

To compile the node, please install [cargo](https://doc.rust-lang.org/cargo/)
and [rustup](https://rustup.rs/), then run the following commands:

    sudo apt update
    sudo apt install -y build-essential pkg-config libzmq3-dev libssl-dev libpq-dev cmake
    rustup default nightly
    git clone https://github.com/LNP-BP/rgb-node.git
    cd rgb-node
    cargo build --release

Now, to run the node you can execute

    target/release/rgbd --data-dir ~/.rgb --bin-dir target/release -vvvv - contract fungible

### In docker

In order to build and run a docker image of the node, run:
```bash
docker build -t rgb-node .
docker run --rm --name rgb_node rgb-node
```

## Using

First, you need to start daemons:
`rgbd -vvvv -d <data_dir> -b <bin_dir>, --contract fungible`
where `bin_dir` is a directory with all daemons binaries (usually `target/debug`
from repo source after `cargo build --bins` command).

Issuing token:
`rgb-cli -d <data_dir> -vvvv fungible issue TCKN "SomeToken" <supply>@<txid>:<vout>`

Next, list your tokens
`rgb-cli -d <data_dir> -vvvv fungible list`

Do an invoice
`rgb-cli -d <data_dir> -vvvv fungible invoice <contract_id> <amount> <txid>:<vout>`,
where `<contract_id>` is id of your token returned by the last call, and
`<txid>:<vout>` must be a transaction output you are controlling.

Save the value of the binding factor you will receive: it will be required in
the future to accept the transfer. Do not share it!
Send the invoice string to the payee.

Doing transfer: this requires preparation of PSBT; here we use ones from our 
sample directory
`rgb-cli -d <data_dir> -vvvv fungible transfer "<invoice>" test/source_tx.psbt 1 <consignment_file> test/dest_tx.psbt -i <input_utxo> [-a <amount>@<change_utxo>]`
NB: input amount must be equal to the sum of invoice amount and change amounts.

This will produce consignment. Send it to the receiving party.

The receiving party must do the following:
`rgb-cli -d <data_dir> -vvvv fungible accept <consignment_file> <utxo>:<vout> <blinding>`,
where `utxo` and the `blinding` must be values used in invoice generation

## Language bindings

The following bindings are available:
- [Android](/ffi/android)
- [iOS](/ffi/ios)
- [Node.js](/ffi/nodejs)

## Developer guidelines

In order to update the project dependencies, run `cargo update`.
If any dependency updates, the `Cargo.lock` file will be updated, keeping
track of the exact package version.
After an update, run tests (`cargo test`) and manually test the software
in order to stimulate function calls from updated libraries.
If any problem arises, open an issue.
