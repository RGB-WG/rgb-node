# Node.js bindings

## Build

In order to build Node.js bindings, from the project root run:

### Local

```bash
sudo apt install -y swig node-gyp
cd ffi
cargo build --release
cd nodejs
curl -o- https://raw.githubusercontent.com/creationix/nvm/v0.34.0/install.sh | bash
nvm install v10
npm install
node example.js
```

### In docker

```bash
docker build -f ffi/nodejs/Dockerfile -t rgb-nodejs .
docker run -it --rm -v $(pwd):/opt/mount --entrypoint cp \
    rgb-nodejs \
    /rgb-node/target/debug/librgb.so /rgb-node/rgb_node.node /opt/mount/
```
