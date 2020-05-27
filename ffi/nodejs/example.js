const ex = require('./build/Release/rgb_node');

const config = {
    network: "bitcoin",
    stash_endpoint: "ipc:{data_dir}/{network}/stashd.rpc",
    contract_endpoints: {
        Fungible: "ipc:{data_dir}/{network}/fungibled.rpc"
    }
};

ex.start_rgb(JSON.stringify(config))
    .then(r => ex.issue(r, JSON.stringify({
        network: "bitcoin", 
        ticker: "USDT",
        name: "USD Tether",
        issue_structure: "SingleIssue",
        precision: 0,
    })))
    .catch(e => console.log(e));
