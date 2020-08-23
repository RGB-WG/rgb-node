
Blinding 4038784059
d6fa3ee02d33c1a53bd055bbd08de65f0ad3acdebfc11854d1aedad96a9dcf51:0

    rgb-cli -vvvv -d ./data \
        fungible transfer \
        'rgb20:txo1zagrm3jewnstxffu7zpmphct06z2r3gs4756rsc2rsamzsrg3sfqqxp4rg?asset=rgb1509cudjhex0qyae5z205zvz6t0g8k0d0j2m7e00zgp8fwkvena8q2wndg4&amount=100' \
        test/source_tx.psbt \
        1 a89be7f38c23a87b47f3639cd8a7264c0397a4256d85e785d45adb12e1deb9e6:0 \
        test/consignment.rgb test/dest_tx.psbt \
        -i e21e06f1c6603a23053c325d06b8abf617f65de061443c3c5f2da0a93ca1d49d:0
    
    
    rgb-cli -vvvv -d ./data fungible accept test/consignment.rgb d6fa3ee02d33c1a53bd055bbd08de65f0ad3acdebfc11854d1aedad96a9dcf51:0 4038784059
    
[2020-07-04T14:05:29Z TRACE lnpbp::rgb::stash::anchor] Preparing anchors with source data {4e9f9959974e40e2bdecb792af3d7bd05b5a30419f123477029ec957368ecba3: ffe62134e3d3185239473222460c6320259f685a80a95f46926c060058a2c87d}
[2020-07-04T14:05:29Z TRACE lnpbp::rgb::stash::anchor] Anchors computed data: {0: {a3cb8e3657c99e027734129f41305a5bd07b3daf92b7ecbde2404e9759999f4e: 7dc8a25800066c92465fa9805a689f2520630c46223247395218d3e33421e6ff}}
[2020-07-04T14:05:29Z TRACE lnpbp::lnpbps::lnpbp4] Resulting commitment string: [MultimsgCommitmentItem { protocol: None, commitment: 8e6b4083c1e8c1cb6c90bc8f16231de20797175efc0dbe96579bfac72aafdd3b }, MultimsgCommitmentItem { protocol: Some(a3cb8e3657c99e027734129f41305a5bd07b3daf92b7ecbde2404e9759999f4e), commitment: 7dc8a25800066c92465fa9805a689f2520630c46223247395218d3e33421e6ff }, MultimsgCommitmentItem { protocol: None, commitment: b37dacf92578777c8b9442682b06fbf8e8951b034d708336e2073b60780d4149 }]
[2020-07-04T14:05:29Z TRACE lnpbp::rgb::stash::anchor] Anchor for output 0: MultimsgCommitment { commitments: [MultimsgCommitmentItem { protocol: None, commitment: 8e6b4083c1e8c1cb6c90bc8f16231de20797175efc0dbe96579bfac72aafdd3b }, MultimsgCommitmentItem { protocol: Some(a3cb8e3657c99e027734129f41305a5bd07b3daf92b7ecbde2404e9759999f4e), commitment: 7dc8a25800066c92465fa9805a689f2520630c46223247395218d3e33421e6ff }, MultimsgCommitmentItem { protocol: None, commitment: b37dacf92578777c8b9442682b06fbf8e8951b034d708336e2073b60780d4149 }], entropy: Some(10213925547421597963) }
