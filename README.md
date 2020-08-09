# RGB Node & SDK

[![TravisCI](https://api.travis-ci.com/LNP-BP/rgb-node.svg?branch=master)](https://api.travis-ci.com/LNP-BP/rgb-node)

This repository contains RGB node source code and SDK for wallet & server-side
development.

The node may run as a set of daemons (even in different docker containers);
a multi-threaded single process or as a set of managed threads within a
wallet app.

To compile the node, please install [cargo](https://doc.rust-lang.org/cargo/) and [rustup](https://rustup.rs/), then run the following commands:

    sudo apt update
    sudo apt install -y build-essential pkg-config libzmq3-dev libssl-dev libpq-dev cmake
    rustup default nightly
    git clone https://github.com/LNP-BP/rgb-node.git
    cd rgb-node
    cargo build --release

Now, to run the node you can execute

    target/release/rgbd --data-dir ~/.rgb --bin-dir target/release -v -v -v -v

If you need NodeJS integration, you have to do the following:
    
    sudo apt install -y swig node-gyp
    cd ffi
    cargo build --release
    cd nodejs
    curl -o- https://raw.githubusercontent.com/creationix/nvm/v0.34.0/install.sh | bash
    nvm install v10
    npm install
    node example.js
