# RGB Node & SDK

[![TravisCI](https://api.travis-ci.com/LNP-BP/rgb-node.svg?branch=master)](https://api.travis-ci.com/LNP-BP/rgb-node)

This repository contains RGB node source code and SDK for wallet & server-side
development.

The node may run as a set of daemons (even in different docker containers);
a multi-threaded single process or as a set of managed threads within a
wallet app.

## Usage

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

    target/release/rgbd --data-dir ~/.rgb --bin-dir target/release -v -v -v -v

### In docker

In order to build and run a docker image of the node, run:
```bash
docker build -t rgb-node .
docker run --rm --name rgb_node rgb-node
```

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
