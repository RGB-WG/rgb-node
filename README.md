# librgb
`librgb` is the reference implementation of [RGB Protocol](https://github.com/rgb-org/spec).

`librgb` is written in "Rust"; "Cargo" is its build system and package manager.

## Install "Rust" and "Cargo"

Follow the instructions in [Rust Install](https://www.rust-lang.org/en-US/install.html)
For those who use "macOS" it is possible to install "Rust" through `brew`:

`$ brew install rust`

## Build `librgb`

`$ cargo build`

## Example "main"

`examples/main.rs` is an example, using the `librgb` built above.

* build: `$ cargo build --example main`
* run: `$ cargo run --example main`
