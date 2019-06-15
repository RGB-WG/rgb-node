[![Build Status](https://travis-ci.org/dr-orlovsky/rgb-rust.svg?branch=master)](https://travis-ci.org/dr-orlovsky/rgb-rust) [![Codacy Badge](https://api.codacy.com/project/badge/Grade/6289725dbd8d4751b3fa8180e962c185)](https://www.codacy.com/app/dr-orlovsky/rgb-rust?utm_source=github.com&amp;utm_medium=referral&amp;utm_content=dr-orlovsky/rgb-rust&amp;utm_campaign=Badge_Grade) [![Coverage Status](https://coveralls.io/repos/github/dr-orlovsky/rgb-rust/badge.svg?branch=master)](https://coveralls.io/github/dr-orlovsky/rgb-rust?branch=master)

# RGB library on Rust

This is re-implementation of https://github.com/rgb-org/rgb according to the most recent spec 
[RGB Protocol](https://github.com/rgb-org/spec) with better test coverage and attention to the details.

`rgb-rust` is written in "Rust"; "Cargo" is its build system and package manager.

## Install "Rust" and "Cargo"

Follow the instructions in [Rust Install](https://www.rust-lang.org/en-US/install.html)
For those who use "macOS" it is possible to install "Rust" through `brew`:

`$ brew install rust`

## Build `rgb-rust`

`$ cargo build`

## Run the tests

`$ cargo test --package rgb --lib tests`
