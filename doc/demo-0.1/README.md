RGB-NODE DEMO
===

### Introduction
This document contains a textual version of the [rgb-node demo]( https://www.youtube.com/watch?v=t_EtUf4601A). It is meant to demonstrate rgb-node's functionality and interface as for [version 0.1.1](https://github.com/LNP-BP/rgb-node/releases/tag/v0.1.1).

Two different setups are available:
- [local installation](#local)
- [docker](#docker)

Once either of them is complete, you can proceed with the actual [demo](#demo)

## Local

#### Requirements
- [cargo](https://doc.rust-lang.org/book/ch01-01-installation.html#installation)
- [git](https://git-scm.com/downloads)

Furthermore, you will need to install a number of system dependencies:
```bash=
sudo apt install -y build-essential pkg-config libzmq3-dev libssl-dev libpq-dev libsqlite3-dev cmake
```
### Build & Run
We can proceed with the installation:
```bash=
git clone https://github.com/LNP-BP/rgb-node.git
cd rgb-node/doc/demo-0.1
cargo install --locked --root . rgb_node --version 0.1.1
```
And then run a couple of nodes into separate terminals (use *ctrl+C* or `kill $(pgrep rgbd)` to stop them):
```bash=
./bin/rgbd -vvvv -b ./bin -d ./data0
./bin/rgbd -vvvv -b ./bin -d ./data1
```
and setup aliases to ease calls to command-line interfaces:
```bash=
alias rgb0-cli="./bin/rgb-cli -d ./data0"
alias rgb1-cli="./bin/rgb-cli -d ./data1"
```

## Docker

#### Requirements
- [git](https://git-scm.com/downloads)
- [docker](https://docs.docker.com/get-docker/)
- [docker-compose](https://docs.docker.com/compose/install/)

### Build & Run
Clone the repository and run a couple of nodes in docker containers:
```bash=
git clone https://github.com/LNP-BP/rgb-node.git
cd rgb-node/doc/demo-0.1
# run containers (it takes a while for the first time since docker image is also built...)
docker-compose up [-d]
```
To get their respective logs you can run, for instance:
```bash=
docker-compose logs [-f] rgb-node-0
```
*Note:* the persistency of some data is obtained through docker volumes. In order to start the demo from scratch you can run `docker-compose down -v`.

*Note:* to stop/start/restart containers you can use `docker-compose stop/start/restart`

Finally we can setup aliases to ease calls to command-line interfaces:
```bash=
alias rgb0-cli="docker exec -it rgb-node-0 rgb-cli"
alias rgb1-cli="docker exec -it rgb-node-1 rgb-cli"
```

## Demo
In this demo, `rgb-node-0` acts as an issuer and transfers some of the newly minted asset to the user, `rgb-node-1`.

In order to get an idea of the functionality exposed by `rgb-cli`, you can run for instance:
```bash=
rgb0-cli help
rgb0-cli fungible help
rgb0-cli fungible help list
rgb0-cli genesis help
```
### Premise

RGB-node does not handle wallet-related functionality, it just performs RGB-specific tasks over data that will be provided by an external wallet such as [bitcoind](https://github.com/bitcoin/bitcoin). In particular, in order to demonstrate a basic workflow with issuance and transfer, we will need:
- an `issuance_utxo` to which `rgb-node-0` will bind the newly issued asset
- a `change_utxo` on which `rgb-node-0` receives the asset change
- a `receive_utxo` on which `rgb-node-1` receives the asset
- a partially signed bitcoin transaction (`transfer_psbt`), whose output pubkey will be tweaked to include a commitment to the transfer.

For the purposes of this demo, since we are skipping the blockchain verification part, you can use "fake" data generated with a testnet or regtest bitcoin node. The following hardcoded utxos (that will be used later) will work:

- `issuance_utxo`: `5aa2d0a8098371ee12b4b59f43ffe6a2de637341258af65936a5baa01da49e9b:0`
- `change_utxo`: `0c05cea88d0fca7d16ed6a26d622e7ea477f2e2ff25b9c023b8f06de08e4941a:1`
- `receive_utxo`: `79d0191dab03ffbccc27500a740f20a75cb175e77346244a567011d3c86d2b0b:0`
- an example `transfer_psbt` can be found in the `doc/demo-0.1/samples` folder

### Asset issuance
To issue an asset, run:
```bash=
rgb0-cli fungible issue USDT "USD Tether" 1000@5aa2d0a8098371ee12b4b59f43ffe6a2de637341258af65936a5baa01da49e9b:0
# no output. rgb-node-0 logs, among other things:
# [2020-11-19T08:52:46Z DEBUG rgb::contracts::fungibled::runtime] Got ISSUE Issue { ticker: "USDT", title: "USD Tether", description: None, supply: None, inflatable: None, precision: 0, allocate: [Outcoins { coins: 1000.0, vout: 0, txid: Some(5aa2d0a8098371ee12b4b59f43ffe6a2de637341258af65936a5baa01da49e9b) }] }
# [2020-11-19T08:52:46Z DEBUG rgb::contracts::fungibled::runtime] API request processing complete
```
This will create a new genesis that includes asset metadata and the allocation of the initial amount to the `<issuance_utxo>`. You can look into it by running:
```bash=
# retrieve <contract-it> with:
rgb0-cli genesis list
# export the genesis contract (use -f to select output format)
rgb0-cli genesis export <contract-id>
```
You can list known fungible assets with:
```bash=
rgb0-cli fungible list
# example output:
# ---
# - name: USD Tether
#   ticker: USDT
#   id: rgb1yprwyam35r0varfhtswcawkhwdhm025mzd5qcejeun0kqj0dtqyqpjn5w0
```
which also outputs its `asset-id`, which is needed to create invoices.

### Generate invoice
In order to receive the new USDT, `rgb-node-1` needs to generate an invoice for it:
```bash=
rgb1-cli fungible invoice <asset-id> 100 79d0191dab03ffbccc27500a740f20a75cb175e77346244a567011d3c86d2b0b:0
# example output:
# [...]
# Invoice: rgb20:utxob10gz0mmkpn2jykqkdwpjeltfwptkfke00zsgmz8lvu7ffsschq0sqxwjnq4?asset=rgb1yprwyam35r0varfhtswcawkhwdhm025mzd5qcejeun0kqj0dtqyqpjn5w0&amount=100
# Outpoint blinding factor: 6754896042118498142
```
To be able to accept transfers related to this invoice, we will need the original `receive_utxo` and the `blinding_factor` that was used to include it in the invoice.

### Transfer
To transfer some amounts of the asset to `rgb-node-1` to pay the new invoice, `rgb-node-0` needs to create a consignment and commit to it into a bitcoin transaction. So we will need the invoice and a partially signed bitcoin transaction that we will modify to include the commitment. Furthermore, `-i` and `-a` options allow to provide an input utxo from which to take the asset and an allocation for the change in the form `<amount>@<utxo>`.

```bash=
# NB: pass the invoice between quotes to avoid misinterpretation of the & character it contains
rgb0-cli fungible transfer '<invoice>' samples/source_tx.psbt samples/consignment.rgb samples/witness.psbt -i 5aa2d0a8098371ee12b4b59f43ffe6a2de637341258af65936a5baa01da49e9b:0 -a 900@0c05cea88d0fca7d16ed6a26d622e7ea477f2e2ff25b9c023b8f06de08e4941a:1
# example output:
# [...]
# Transfer succeeded, consignment data are written to "samples/consignment.rgb", partially signed witness transaction to "samples/witness.psbt"
```
This will write the consignment file and the psbt including the tweak (which is called *witness transaction*) at the provided paths.

At this point the witness transaction should be signed and broadcasted, while the consignment is sent offchain to the peer. For the purpose of the demo, both nodes have access to the consignment file at the same path (in docker this is obtained through a [volume](https://docs.docker.com/storage/volumes/))

### Accept
To accept an incoming transfer you need to provide `rgb-node-1` with the consignment file received from `rgb-node-0`, the `receive_utxo` and the corresponding `blinding_factor` that were defined at invoice creation.
```bash=
rgb1-cli fungible accept samples/consignment.rgb 79d0191dab03ffbccc27500a740f20a75cb175e77346244a567011d3c86d2b0b:0 <blinding_factor>
# example output:
# Asset transfer successfully accepted.
```
Now you are able to see (in the `known_allocations` field) the new allocation of 100 asset units at `<receive_utxo>` by running :
```bash=
rgb1-cli fungible list -l
```
*Note:* since the `receive_utxo` was blinded inside the invoice, the payer has no information on where the asset was allocated after the transfer, so the new allocation does not appear in:
```bash=
rgb0-cli fungible list -l
```

