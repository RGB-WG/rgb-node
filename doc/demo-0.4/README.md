RGB-NODE DEMO
===

## Introduction
This directory contains an rgb-node demo based on version 0.4.2.
It is an evolution of the original rgb-node demo by
[St333p](https://github.com/St333p) (based on version 0.1) and closely
resembles [grunch](https://github.com/grunch)'s
[guide](https://grunch.dev/blog/rgbnode-tutorial/).

The demo is meant to be run with Docker and uses Rust 1.60.0 and Debian
bullseye. The underlying Bitcoin network is `regtest`.

Commands are to be executed in a bash shell. Example output is provided to
allow following the links between the steps. Actual output when executing the
procedure will be different each time.

Two versions of the demo are available:
- an automated one
- a manual one

The automated version is meant to provide a quick and easy way to see an RGB
token be created and transferred. The manual version is meant to provide a
hands-on experience with an RGB token and gives step-by-step instructions on
how to operate all the required components.

## Setup
Some setup is required before proceeding with the actual demo.

### Requirements
- [git](https://git-scm.com/downloads)
- [docker](https://docs.docker.com/get-docker/)
- [docker-compose](https://docs.docker.com/compose/install/)

### Start services
Clone the repository and start the requires services in Docker containers:
```bash=
git clone https://github.com/LNP-BP/rgb-node.git
cd rgb-node/doc/demo-0.4
# create service data directories
mkdir data{0,1,2,core,index}
# run containers (first time takes a while to download/build docker images...)
docker-compose up -d
```

To get a list of the running services you can run:
```bash=
docker-compose ps
```

To get their respective logs you can run, for instance:
```bash=
docker-compose logs rgb-node-0
```

In order to clean up containers and data to start the demo from scratch, run:
```bash=
docker-compose down               # stop and remove running containers
rm -fr data{0,1,2,core,index}     # remove service data directories
```

## Automated version
To check out the automated demo, run:
```bash=
bash demo.sh
```

The automated script will create Bitcoin wallets, generate UTXOs, issue an
asset, transfer some of it from issuer to a first recipient and, finally,
transfer some of the assets received by the first recipient to a second one.
For a deeper look at what's happening during the demo, add the `-v` option
(`bash demo.sh -v`) to enable more verbose output, which shows the commands
being run on nodes and output from additional commands, such as `listunspent`
and `fungible list`.

## Manual version
The manual demo will show how to issue an asset and transfer some tokens to a
recipient.

### Premise
RGB-node does not handle wallet-related functionality, it just performs
RGB-specific tasks over data that will be provided by an external wallet such
as [Bitcoin Core](https://github.com/bitcoin/bitcoin). In particular, in order
to demonstrate a basic workflow with issuance and transfer, we will need:
- an *issuance_utxo* to which `rgb-node-0` will bind the newly issued asset
- a *change_utxo* on which `rgb-node-0` receives the asset change
- a *receive_utxo* on which `rgb-node-1` receives the asset
- a partially signed bitcoin transaction (`tx.psbt`), whose output pubkey
  will be tweaked to include a commitment to the transfer

### Demo
We setup aliases to ease calls to command-line interfaces:
```bash=
alias bcli='docker-compose exec -u blits bitcoind bitcoin-cli -regtest'
alias rgb0-cli='docker-compose exec -u rgbd rgb-node-0 rgb-cli -n regtest'
alias rgb1-cli='docker-compose exec -u rgbd rgb-node-1 rgb-cli -n regtest'
```

We prepare UTXOs using Bitcoin Core:
```bash=
# issuer wallet with some UTXOs
bcli createwallet rgbdemo
bcli -generate 103

# receiver wallet
bcli createwallet rgbrcpt
bcli -rpcwallet=rgbrcpt getnewaddress
# example output:
# bcrt1qmj970g5a5g3r0sunnsl54q2ce52mlwacf8exr2

# send some coins to create a UTXO on the receiver wallet
bcli -rpcwallet=rgbdemo sendtoaddress bcrt1qmj970g5a5g3r0sunnsl54q2ce52mlwacf8exr2 2
# example output:
# ad3ebdcda0f83b37fffab0439c89fd3ef7d99c41c353a45a98d5983d9ad00183
bcli -rpcwallet=rgbdemo -generate 1

# list and collect UTXOs for the demo
bcli -rpcwallet=rgbdemo listunspent
# example output:
# [
#   {
#     "txid": "29a9670f082f661aff4af1e3be5eee8a7ced58981e133afe74f2aa43dbc95b06",
#     "vout": 0,
#     "address": "bcrt1qd54ht84lapr5jqyk0hs2t4g5pmjhqtlf99hcys",
#     "label": "",
#     "scriptPubKey": "00146d2b759ebfe8474900967de0a5d5140ee5702fe9",
#     "amount": 50.00000000,
#     "confirmations": 104,
#     "spendable": true,
#     "solvable": true,
#     "desc": "wpkh([399b98af/0'/0'/0']027b7ce4035b99983f830fb35e2bccca63d6b30f7ffe02a44ab6caa5d572582a8c)#6a4az9sk",
#     "safe": true
#   },
#   {
#     "txid": "c8fec82dc4b3edaba5ad979f3e9d897d05e4dfe3c8bca9e995a77ceeaa4a9709",
#     "vout": 0,
#     "address": "bcrt1qd54ht84lapr5jqyk0hs2t4g5pmjhqtlf99hcys",
#     "label": "",
#     "scriptPubKey": "00146d2b759ebfe8474900967de0a5d5140ee5702fe9",
#     "amount": 50.00000000,
#     "confirmations": 102,
#     "spendable": true,
#     "solvable": true,
#     "desc": "wpkh([399b98af/0'/0'/0']027b7ce4035b99983f830fb35e2bccca63d6b30f7ffe02a44ab6caa5d572582a8c)#6a4az9sk",
#     "safe": true
#   },
#   ...
# ]

bcli -rpcwallet=rgbrcpt listunspent
# example output:
# [
#   {
#     "txid": "ad3ebdcda0f83b37fffab0439c89fd3ef7d99c41c353a45a98d5983d9ad00183",
#     "vout": 0,
#     "address": "bcrt1qmj970g5a5g3r0sunnsl54q2ce52mlwacf8exr2",
#     "label": "",
#     "scriptPubKey": "0014dc8be7a29da22237c3939c3f4a8158cd15bfbbb8",
#     "amount": 2.00000000,
#     "confirmations": 1,
#     "spendable": true,
#     "solvable": true,
#     "desc": "wpkh([70ced5e7/0'/0'/0']036666df6b3a21475bf84812bb329948c2c251de88a7cb30be8b8660f8494203c3)#gwawk2q5",
#     "safe": true
#   }
# ]
```

From the above setup we get the following UTXOs:
- *issuance_utxo*: `29a9670f082f661aff4af1e3be5eee8a7ced58981e133afe74f2aa43dbc95b06:0`
- *change_utxo*: `c8fec82dc4b3edaba5ad979f3e9d897d05e4dfe3c8bca9e995a77ceeaa4a9709:0`
- *receive_utxo*: `ad3ebdcda0f83b37fffab0439c89fd3ef7d99c41c353a45a98d5983d9ad00183:0`

#### Asset issuance
To issue an asset, run:
```bash=
rgb0-cli fungible issue USDT "USD Tether" 1000@29a9670f082f661aff4af1e3be5eee8a7ced58981e133afe74f2aa43dbc95b06:0
# example output:
# Asset successfully issued. Use this information for sharing:
# Asset information:
#  ---
# genesis: genesis1qyfe883hey6jrgj2xvk5g3dfmfqfzm7a4wez4pd2krf7ltsxffd6u6nrvjvv8s9xjnnjv2x9r4uk402sste7h68ar2rlmt0z3j67frm0mfh08vtryglnks63dfayj6n3exhl6km0cyu5crp9f94l84hyxyerqvyqxzp3yycrrzcrp9pcs5u0xvyyqq2jjwr9jcjwwelxg9uutzf9l8z3nxgvp7lm9qj5z2eps6pw90v8g3gksmgxp9csgcgfxz7y2syyjtwffqkkypkmespzfwdapgffxsrx9cq89xzlxppeeskgkzggryge5nk7474nc5h62nr0kdzel08dhn498jfl9dsmjxg3da4m4hk9a4ale50tha2f50s80lay5nvs96u8vqsqa34sq4hudh3
# id: rgb1kksqr2rk6zxa42cx7mqe5xdsuuc97xt420valkwvyvzex3z5u9ts92l5jl
# ticker: USDT
# name: USD Tether
# description: ~
# knownCirculating: 1000
# isIssuedKnown: ~
# issueLimit: 0
# chain: regtest
# decimalPrecision: 0
# date: "2022-04-21T14:55:06"
# knownIssues:
#   - id: 57e15444930523ccd9dfd95375195f30e7b0199ac1f606abda8dd076a801a0b5
#     amount: 1000
#     origin: ~
# knownInflation: {}
# knownAllocations:
#   - nodeId: 57e15444930523ccd9dfd95375195f30e7b0199ac1f606abda8dd076a801a0b5
#     index: 0
#     outpoint: "29a9670f082f661aff4af1e3be5eee8a7ced58981e133afe74f2aa43dbc95b06:0"
#     revealedAmount:
#       value: 1000
#       blinding: "0000000000000000000000000000000000000000000000000000000000000001"
```
This will create a new genesis that includes asset metadata and the allocation
of the initial amount to the *issuance_utxo*.

You can list known fungible assets with:
```bash=
rgb0-cli fungible list
# example output:
# ---
# - name: USD Tether
#   ticker: USDT
#   id: rgb1yprwyam35r0varfhtswcawkhwdhm025mzd5qcejeun0kqj0dtqyqpjn5w0
```
which also outputs the asset ID, needed to make the transfer.

#### Generate blinded UTXO
In order to receive the new asset, `rgb-node-1` needs to provide `rgb-node-0`
with a *blinded_utxo*. To do so we blind the *receive_utxo*:
```bash=
rgb1-cli fungible blind ad3ebdcda0f83b37fffab0439c89fd3ef7d99c41c353a45a98d5983d9ad00183:0
# example output:
# Blinded outpoint: utxob1kewrvnf8sjmarq65gv98lz2xrgxylpnlta8lc3p78fjxaw9qda4qkewlwr
# Outpoint blinding secret: 8114079862469528952
```
This also gives us the *blinding_secret* that will be needed later on to accept
transfers related to this UTXO.

#### Transfer
To transfer some amounts of the asset to the `rgb-node-1` *blinded_utxo*,
`rgb-node-0` needs to create a consignment and disclosure, and commit to it
into a bitcoin transaction. So we will need a partially signed bitcoin
transaction that we will modify to include the commitment.

Generate a new address for the bitcoin (non-asset) portion of the transaction:
```bash=
bcli -rpcwallet=rgbdemo getnewaddress
# example output:
# bcrt1q3eg9yffkfmkzkapxd0f9xu7y4p6eysnjw89d5z
```

Create the initial PSBT, specifying the *issuance_utxo* as input and the freshly
generated address as output, using the correct amount, then write it to a file and
make it available to `rgb-node-0`:
```bash=
bcli -rpcwallet=rgbdemo walletcreatefundedpsbt '[{"txid": "29a9670f082f661aff4af1e3be5eee8a7ced58981e133afe74f2aa43dbc95b06", "vout": 0}]' '[{"bcrt1q3eg9yffkfmkzkapxd0f9xu7y4p6eysnjw89d5z": "0.1"}]'
# example output:
# {
#   "psbt": "cHNidP8BAHECAAAAAQZbydtDqvJ0/joTHphY7XyK7l6+4/FK/xpmLwgPZ6kpAAAAAAD/////AoCWmAAAAAAAFgAUjlBSJTZO7Ct0JmvSU3PEqHWSQnJ8UG0pAQAAABYAFFgyZ28x19mZuKesFv6Bgkvd4PpcAAAAAAABAIQCAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/////wNRAQH/////AgDyBSoBAAAAFgAUbSt1nr/oR0kAln3gpdUUDuVwL+kAAAAAAAAAACZqJKohqe3i9hw/cdHe/T+pmd+jaVN1XGkGiXmZYrSL69g2l06M+QAAAAABAR8A8gUqAQAAABYAFG0rdZ6/6EdJAJZ94KXVFA7lcC/pIgYCe3zkA1uZmD+DD7NeK8zKY9azD3/+AqRKtsql1XJYKowQOZuYrwAAAIAAAACAAAAAgAAAIgIDEoO+VR+P6ps01i008NJTTw0tnQXCuXhVldeeDJhfHTgQOZuYrwAAAIABAACAAwAAgAA=",
#   "fee": 0.00002820,
#   "changepos": 1
# }

echo "cHNidP8BAHECAAAAAQZbydtDqvJ0/joTHphY7XyK7l6+4/FK/xpmLwgPZ6kpAAAAAAD/////AoCWmAAAAAAAFgAUjlBSJTZO7Ct0JmvSU3PEqHWSQnJ8UG0pAQAAABYAFFgyZ28x19mZuKesFv6Bgkvd4PpcAAAAAAABAIQCAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/////wNRAQH/////AgDyBSoBAAAAFgAUbSt1nr/oR0kAln3gpdUUDuVwL+kAAAAAAAAAACZqJKohqe3i9hw/cdHe/T+pmd+jaVN1XGkGiXmZYrSL69g2l06M+QAAAAABAR8A8gUqAQAAABYAFG0rdZ6/6EdJAJZ94KXVFA7lcC/pIgYCe3zkA1uZmD+DD7NeK8zKY9azD3/+AqRKtsql1XJYKowQOZuYrwAAAIAAAACAAAAAgAAAIgIDEoO+VR+P6ps01i008NJTTw0tnQXCuXhVldeeDJhfHTgQOZuYrwAAAIABAACAAwAAgAA=" | base64 -d > tx.psbt

cp tx.psbt data0/
```
*Note:* this PSBT will typically contain a second output for the bitcoin
change; this is not an issue for the manual demo but in the real case you
should avoid this to save blockchain space and you can do so by getting the
exact bitcoin amount to be spent (looking for the PSBT input transaction in the
wallet's unspent outputs) and subtracting the fee from the amount being spent.

Initiate the asset transfer, providing the *blinded_utxo*, amount, asset ID,
PSBT file name, the names of files to be generated (consignment, disclosure and
updated PSBT), and finally the input (*issuance_utxo*) and change
(amount@*change_utxo*):
```bash=
rgb0-cli fungible transfer utxob1kewrvnf8sjmarq65gv98lz2xrgxylpnlta8lc3p78fjxaw9qda4qkewlwr 100 rgb1kksqr2rk6zxa42cx7mqe5xdsuuc97xt420valkwvyvzex3z5u9ts92l5jl tx.psbt consignment.rgb disclosure.rgb witness.psbt -i 29a9670f082f661aff4af1e3be5eee8a7ced58981e133afe74f2aa43dbc95b06:0 -a 900@c8fec82dc4b3edaba5ad979f3e9d897d05e4dfe3c8bca9e995a77ceeaa4a9709:0
# example output:
# Transfer succeeded, consignments and disclosure are written to "consignment.rgb" and "disclosure.rgb", partially signed witness transaction to "witness.psbt"
# Consignment data to share:consignment1q93kqyynncmujdfp5f9rxt2ygk5a5sy3dlw6hv32sk4tp5l04cry5kawdf3kfxxrcznffeex9rz367t2h4gg9ultar734pla4h3ged0y3aha5mhnk93jy0emgdgk57jfdfcuntlatdhuzw2vpsj5j6ln6mjrzv3sxzqrpqcjzvp33vpsjsug2w8nxzzqq9ff8pjevf88vlnyz7w93yjln3genyxql0ajsf2p9vscdqhzhkr5g5tgd5rqjugyvyynp0z9gzzf9hy5sttzqmducq3yhx7s5y5ngpnzuqrjnp0nqsuuctytpyypjyv6fm02l2eu2ta9f3hmx3vlhnkme6jneyljkcderygk76a6mmz767lu684m74y68crhl7j2fkgzawrkq2qpzs5uun34l9p27rjwdfhyq5el0nhd4dl0svww3janfe526rr7lx60sn0tyewcze37d277kt7kyuugxdthmfumzj8h7kmahnl379em4w2tc8gt72e3sxvveeyjd0na8whkfnwkmklpxfr8m5ll057t0qm6kn9m62pctx6m4jmecq46ulpurwe9wlpjw72wwma22mqevkg7400h2jj6eejkamza6huapeyejuesrn37haxgas7muvhrrgaaxp5hnc8e8djhdwuzukrj5txr8mzdgz2gyv0r6czcc2729m6dukkd7mk7qtxfph8smc39fwpklh4l09rfjadjf0yvxuyzlespv6k7zu4dllm2kmyn2hvnp7tc9ltewhnmy8k6tygn4a068j0pnudjzmg9qgtyzyu9dc93qdxrt75vxavxg4l96h9zl98gpwxnzwdwxfdax744e8usd8e83uhlu73scwp780txmzf9lvsaulhtzxymehwwvayzuk797hkfhc4k6egef8uul0h4eavuee85ut2ela74ndk4y0y08fz660j7c5c5th006kw7f5aa6t0t9yah8sfpx7rnvhhrreak89hlykga30wqum244kyvu7j577aanew5ej7tegy68dxf7rkujuhyc9jlkjluv7nj6gxmjmkt3jmszqtdxdu7lmn07hlan9kmtkd8u0lhmrdev07mmkhw36xwjl47j9vlvpe800srfd54nuv8fvjlxk2ms57mkvenpmcytv6d0nxkr7xl7rdafsmmrldpfmf2mngr642ndhlqcgm77kh02weau0l5ttnahgu9acjd0agy0w82ksxf2ucek0wz95sr9ktvf3y8pru9cag277m0e2dxxal4l4w2puawzkwd24htk86vm4l099lkm64kh6h0ne6ylc8jngf6k542kx5njehmt9ucva2h27y7hpmvhzn8msyk8wsus0jfwhuc8k3f2k2ukf8xnre9swvaf884rkkm8q6kh9l9cyxlwta0dqlt87ae7j3pqfdl82p78f40400jeha4fkhlpkzwp0nggtkecjqxa4kq44q48ksk5nt6wmtttfkhqaypew2gd8rldvxutqsmlzf75h7xxzvp0xsedza0ze0d3yaxk7zdajleh5hfh9e3yp2na69v677xysgrwm2pv8dfd5h97lhpc5pez0ql4u8camtp28aj5674lj86late8qyc4aw0qlhf44d0mm0pm8gm0ec7vhuz6k8qd23tea8ymlul72cajv4laml8f579kuk203zhyan3ukx89ljt4nja4r89550kgz38j6ykn877ryalzlkcunamwn52w206axrv78g80s0jm7hflval64jm2mv47xvzkeca79u4dar2c68z2re08dhhxlc3c7dtwtt2v9yvt3wed603cm87z59pgjca3vxludmpj907ea708mtlatss0xmmeahxmnxqcne0ll07e7gk4tm4u3682cda2h8h4wdz8j9mxdffxvtsf4ath9vunuwfx8h86y82xsuvjx8l9l8y4v28vn0rfrmmxe3e784upp80ptd79caud3t4gv3wuwau7uzn482x9vena7n8jkvmxcy2cllfer3hk7zzfz8e6eu8taw294gkj6r4667kd3kkxgj5pcc2wsqtje5lzt7a2aww5tushka2nlltn4l398f0cnv4dacuj6gx6ep7tpml04sf3q4qqwvndwy
```
This will write the consignment, disclosure and PSBT files.

#### Validate
Before a transfer can be accepted, it needs to be validated. To validate an
incoming transfer you need to provide the recipient (`rgb-node-1`) with the
consignment file produced by the sender (`rgb-node-0`). For the purpose of the
demo, the consignment file is copied between the node data directories:

```bash=
cp data0/consignment.rgb data1/

rgb1-cli fungible validate consignment.rgb
# example output:
# Asset transfer validation report:
# Status {
#     unresolved_txids: [
#         85c84b88...b73c0d26,
#     ],
#     failures: [],
#     warnings: [],
#     info: [],
# }
```
The transfer is valid if no `failures` are reported. It is normal at this stage
for a transaction to show up in the `unresolved_txids` list, as that's the
transaction that has not yet been broadcast, as the sender is waiting for
approval from the recipient.

At this point the recipient approves the transfer (for the demo let's just
assume it happened) and so the witness transaction can be signed and
broadcast:
```bash=
base64 -w0 data0/witness.psbt
# example output:
# cHNidP8BAHECAAAAAQZbydtDqvJ0/joTHphY7XyK7l6+4/FK/xpmLwgPZ6kpAAAAAAD/////AoCWmAAAAAAAFgAUjlBSJTZO7Ct0JmvSU3PEqHWSQnJ8UG0pAQAAABYAFF1RPsap8rgafclRG0USpFNXS7orAAAAAAABAIQCAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/////wNRAQH/////AgDyBSoBAAAAFgAUbSt1nr/oR0kAln3gpdUUDuVwL+kAAAAAAAAAACZqJKohqe3i9hw/cdHe/T+pmd+jaVN1XGkGiXmZYrSL69g2l06M+QAAAAABAR8A8gUqAQAAABYAFG0rdZ6/6EdJAJZ94KXVFA7lcC/pIgYCe3zkA1uZmD+DD7NeK8zKY9azD3/+AqRKtsql1XJYKowQOZuYrwAAAIAAAACAAAAAgAAAIgIDEoO+VR+P6ps01i008NJTTw0tnQXCuXhVldeeDJhfHTgQOZuYrwAAAIABAACAAwAAgAb8A1JHQgEhAxKDvlUfj+qbNNYtNPDSU08NLZ0Fwrl4VZXXngyYXx04BvwDUkdCAiCjXKPrAT3ASLPciBeXHf40dstIP5QmElTvsMCLl9ZxzQA=

bcli -rpcwallet=rgbdemo finalizepsbt $(bcli -rpcwallet=rgbdemo walletprocesspsbt $(base64 -w0 data0/witness.psbt) |jq -r '.psbt')
# example output:
# {
#   "hex": "02000000000101065bc9db43aaf274fe3a131e9858ed7c8aee5ebee3f14aff1a662f080f67a9290000000000ffffffff0280969800000000001600148e505225364eec2b74266bd25373c4a8759242727c506d29010000001600145d513ec6a9f2b81a7dc9511b4512a453574bba2b024730440220336bac192cec26d7bef2ee7ef321c58d5d8a409db9e8b563d3f5a3968d7dd8c7022017fe6e096458d90494eb422d51f74882a8aaf1b51df4f74cb0c11c70ac63148e0121027b7ce4035b99983f830fb35e2bccca63d6b30f7ffe02a44ab6caa5d572582a8c00000000",
#   "complete": true
# }

bcli -rpcwallet=rgbdemo sendrawtransaction 02000000000101065bc9db43aaf274fe3a131e9858ed7c8aee5ebee3f14aff1a662f080f67a9290000000000ffffffff0280969800000000001600148e505225364eec2b74266bd25373c4a8759242727c506d29010000001600145d513ec6a9f2b81a7dc9511b4512a453574bba2b024730440220336bac192cec26d7bef2ee7ef321c58d5d8a409db9e8b563d3f5a3968d7dd8c7022017fe6e096458d90494eb422d51f74882a8aaf1b51df4f74cb0c11c70ac63148e0121027b7ce4035b99983f830fb35e2bccca63d6b30f7ffe02a44ab6caa5d572582a8c00000000
# example output:
# cab43b84b3a1e022871c05bb334af9cedf9a19904dbbb5ac749eeded6119ce32

bcli -rpcwallet=rgbdemo -generate 1
```

#### Accept
Once the transaction has been confirmed it's time to accept the incoming
transfer.

##### Receiver side
Let's first validate the transfer again and confirm the transaction ID is now
unresolved:
```bash=
rgb1-cli fungible validate consignment.rgb
# example output:
# Asset transfer validation report:
# Status {
#     unresolved_txids: [],
#     failures: [],
#     warnings: [],
#     info: [],
# }
```

To accept the transfer you need to provide `rgb-node-1` with the consignment
file received from `rgb-node-0`, plus the *receive_utxo* and the corresponding
*blinding_secret* generated during UTXO blinding:
```bash=
rgb1-cli fungible accept consignment.rgb ad3ebdcda0f83b37fffab0439c89fd3ef7d99c41c353a45a98d5983d9ad00183:0 8114079862469528952
# example output:
# Asset transfer successfully accepted.
```

Now you are able to see (in the `known_allocations` field) the new allocation
of 100 asset units at *receive_utxo* by running `rgb1-cli fungible list -l`.

##### Sender side
Since the *receive_utxo* was blinded during the transfer, the payer has
no information on where the asset was allocated after the transfer, so the new
allocation does not appear in the output of `rgb1-cli fungible list -l`.

This will also not display the change allocation previously provided to the
`fungible transfer` command using the `-a` flag, yet.
To register the change allocation, the disclosure needs to be accepted by
`rgb-node-0`. Ideally this should be done after the witness transaction
(provided in `witness.psbt`) has received a sufficient number of confirmations:
```bash=
rgb0-cli fungible enclose disclosure.rgb
# example output:
# Disclosure data successfully enclosed.
```

After enclosing of `disclosure.rgb` the change allocation of the 900 asset units
will be visible to `rgb-node-0` when running `rgb1-cli fungible list -l`.
