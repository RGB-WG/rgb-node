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
- **RPC**: reactor-based thread managing incoming client connections,
  notifying them about changes to the subscribed information;
- ...

By default, the node exposes a binary RPC API over TCP, which can be exposed as more high-level APIs
(HTTP REST, Websocket-based, or JSON-RPC) using special adaptor services.

## Building clients

RGB Node can be run in two modes:
as a standalone server and as a multithreaded service embedded into some other process.

### Standalone server

The mode is turned on using `server` feature flag and leads to production of `rgbd` executable.
The binary can be run as a daemon, and accessed via binary RGB RPC interface.
The use of the RPC API can be simplified through `rgb-client` crate, providing high-level API.

### Embedded service

The mode is turned on using `embedded` feature flag and leads to production of `rgbnode` library.
To run RGB Node inside any app in this mode please call `Broker::start_embedded`,
giving in the method arguments a configuration and instantiated persistence provider.
The method returns `Broker` instance; call `Broker::client` to receive an `AsyncClient` which can
be used to process all calls to the node.

NB: Do not forget to join the `Broker` thread from the main app by calling `Broker::run` method.

## OS Support

The project currently supports Linux, macOS, and UNIX.
Windows support is a work-in-progress, requiring downstream [`io-reactor`] framework changes.

[`io-reactor`]: https://github.com/rust-amplify/io-reactor

[`microservices.rs`]: https://github.com/cyphernet-labs/microservices.rs

[`netservices.rs`]: https://github.com/cyphernet-labs/netservices.rs
