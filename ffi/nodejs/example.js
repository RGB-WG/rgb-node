const ex = require('./build/Release/rgb_node');

const config = {
    network: "testnet",
    stash_endpoint: "ipc:/tmp/rgb-node/testnet/stashd.rpc",
    contract_endpoints: {
        Fungible: "ipc:/tmp/rgb-node/testnet/fungibled.rpc"
    },
    threaded: false,
    datadir: "/tmp/rgb-node/"
};

ex.start_rgb(JSON.stringify(config))
    /*.then(r => ex.issue(r, JSON.stringify({
        network: "testnet",
        ticker: "USDT",
        name: "USD Tether",
        issue_structure: "SingleIssue",
        allocations: [{ coins: 100, vout:0, txid: "0313ba7cfcaa66029a1a63918ebc426259f00953016c461663315d1bf6b83ab4" }],
        precision: 0,
    })))*/
    .then(r => ex.transfer(r, JSON.stringify({
        inputs: ["0313ba7cfcaa66029a1a63918ebc426259f00953016c461663315d1bf6b83ab4:0"],
        allocate: [{ coins: 100, vout:1, txid: "0313ba7cfcaa66029a1a63918ebc426259f00953016c461663315d1bf6b83ab4" }],
        invoice: "rgb20:outpoint1mzu8vz3jly3rzzkdpph583yahv9wktljtfcln6pe2le6n7ehqulstu967t?amount=5&asset=rgb:id1yqqqxya60n725eszngdx8yvwh3pxyk0sp9fszmzxze3nzhgm76ur4dqf2f7gy",
        prototype_psbt: "cHNidP8BAFICAAAAAZ38ZijCbFiZ/hvT3DOGZb/VXXraEPYiCXPfLTht7BJ2AQAAAAD/////AfA9zR0AAAAAFgAUezoAv9wU0neVwrdJAdCdpu8TNXkAAAAATwEENYfPAto/0AiAAAAAlwSLGtBEWx7IJ1UXcnyHtOTrwYogP/oPlMAVZr046QADUbdDiH7h1A3DKmBDck8tZFmztaTXPa7I+64EcvO8Q+IM2QxqT64AAIAAAACATwEENYfPAto/0AiAAAABuQRSQnE5zXjCz/JES+NTzVhgXj5RMoXlKLQH+uP2FzUD0wpel8itvFV9rCrZp+OcFyLrrGnmaLbyZnzB1nHIPKsM2QxqT64AAIABAACAAAEBKwBlzR0AAAAAIgAgLFSGEmxJeAeagU4TcV1l82RZ5NbMre0mbQUIZFuvpjIBBUdSIQKdoSzbWyNWkrkVNq/v5ckcOrlHPY5DtTODarRWKZyIcSEDNys0I07Xz5wf6l0F1EFVeSe+lUKxYusC4ass6AIkwAtSriIGAp2hLNtbI1aSuRU2r+/lyRw6uUc9jkO1M4NqtFYpnIhxENkMak+uAACAAAAAgAAAAAAiBgM3KzQjTtfPnB/qXQXUQVV5J76VQrFi6wLhqyzoAiTACxDZDGpPrgAAgAEAAIAAAAAAACICA57/H1R6HV+S36K6evaslxpL0DukpzSwMVaiVritOh75EO3kXMUAAACAAAAAgAEAAIAA",
        fee: 346,
        change: "0313ba7cfcaa66029a1a63918ebc426259f00953016c461663315d1bf6b83ab4:2",
        consignment_file: "/tmp/rgb-node/output/consignment",
        transaction_file: "/tmp/rgb-node/output/transaction"
    })))
    .catch(e => console.log(e));
