const ex = require('./build/Release/rgb_node');

const config = {
    network: "testnet",
    stash_endpoint: "ipc:/home/orlovsky/repo/rgb-node/data/testnet/stashd.rpc",
    contract_endpoints: {
        Fungible: "ipc:/home/orlovsky/repo/rgb-node/data/testnet/fungibled.rpc"
    }
};

ex.start_rgb(JSON.stringify(config))
    .then(r => ex.issue(r, JSON.stringify({
        network: "bitcoin", 
        ticker: "USDT",
        name: "USD Tether",
        issue_structure: "SingleIssue",
        allocations: [{ coins: 100, vout:0, txid: "0313ba7cfcaa66029a1a63918ebc426259f00953016c461663315d1bf6b83ab4" }],
        precision: 0,
    })))
    .catch(e => console.log(e));
