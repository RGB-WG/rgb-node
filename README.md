# RGB Node: sovereign smart contracts backend

![Build](https://github.com/RGB-WG/rgb-node/workflows/Build/badge.svg)
![Lints](https://github.com/RGB-WG/rgb-node/workflows/Lints/badge.svg)
[![Apache-2 licensed](https://img.shields.io/crates/l/rgb-node)](./LICENSE)

## Components

This repository contains the following crates:

- `rgb-node`: main indexing daemon, which can be used as an embedded multi-thread service,
  or compiled into a standalone binary (`rgbd`);
- `rgb-client`: client to work with the daemon and a command-line utility `rgb-cli`;
- `rgb-rpc`: a shared crate between `rgb-node` and `rgb-client`.

## Node Architecture

The node operates as a set of threads, communicating through Crossbeam channels.
It leverages [`microservices.rs`] and [`netservices.rs`] crates,
which serves as the node non-blocking reactor-based (see [`io-reactor`]) microservice frameworks.

The node daemon has the following components:

- **Broker**, integrating all services and managing their communications;
- **RPC**: reactor-based thread managing incoming client connections, notifying them about changes
  to the subscribed information;
- ...

By default, the node exposes a binary RPC API over TCP, which can be exposed as more high-level APIs
(HTTP REST, Websocket-based or JSON-RPC) using special adaptor services.

## OS Support

The project currently supports Linux, macOS and UNIX.
Windows support is a work-in-progress, requiring downstream [`io-reactor`] framework changes.

[`io-reactor`]: https://github.com/rust-amplify/io-reactor

[`microservices.rs`]: https://github.com/cyphernet-labs/microservices.rs

[`netservices.rs`]: https://github.com/cyphernet-labs/netservices.rs
