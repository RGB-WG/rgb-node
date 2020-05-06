# Kaleidoscope

Kaleidoscope, command-line wallett for Bitcoion and RGB assets

## Installation

1. Install Cargo: `curl -sSf https://static.rust-lang.org/rustup.sh | sh`
2. `rustup install nightly`
3. `rustup default nightly`
4. `sudo apt install pkg-config libssl-dev libzmq3-dev`
5. Build the project: `cargo build`

When the build is completed, the executable will be located at `./target/debug/kaleidoscope`.

For convenience, it can be useful to temporarily add the directory to your `PATH`, like so:

```
export PATH=$(readlink -f ./target/debug):$PATH
```

Make sure that you can now run the executable with:

```
kaleidoscope --version
```
