const ex = require('./build/Release/rgb_node');

ex.start_rgb()
    .then(r => ex.issue(r, JSON.stringify({
        network: "bitcoin", 
        ticker: "USDT",
        name: "USD Tether",
        issue_structure: "SingleIssue",
        precision: 0,
    })))
    .catch(e => console.log(e));
