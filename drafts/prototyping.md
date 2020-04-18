```shell script
kaleidoscope asset list
kaleidoscope asset add <file-with-history>
kaleidoscope send
```


```console
$ kaleidoscope issue --interactive
$ kaleidoscope issue [--signet] [-r --reissue-enable] [-p --precision 0] [-d --dust-limit 1] 10000 usdt "USD Tether" "Description"
```

```bash
$ kaleidoscope send 
```

```bash
$ kaleidoscope import asset_proofs.rgb
Verifying import file format ... success
INFO: Will import USDT assets, rgb1...
Importing assets from `asset_proofs.rgb`:
- 10 USDT on an output deadbeef.....:1
- 100 USDT on an output 
- 1000 USDT on an onowned output ....
Two asset outputs with total balance of 110 USDT were imported
WARNING: Data file contained unowned outputs; potential data privacy leak
```

```bash
$ kaleidoscope export usdt asset_proofs.rgb
```

Old stuff:
```bash
# $ kaleidoscope schema-import fungible-assets.rgbsch
# schema-list
# schema-remove
#
# $ kaleidoscope asset-add usdt.rgba
# asset-list
# asset-remove
# asset-create
# asset-inflate
# asset-prune
# asset-transfer

# $ kaleidoscope asset-id -i pls.yaml --sign key.pem
# Asset id: rgb1...
#
# $ kaleidoscope asset-issue --format json
# $ kaleidoscope asset-issue --interactive
# $ kaleidoscope asset-issue-verify --id rgb1... -f pls.bin --format binary

# $ kaleidoscope send 100 usdt bc1qwqdg6squsna38e46795at95yu9atm8azzmyvckulcc7kytlcckxswvvzej
# detected asset class: rgb1.....
# detected unspent outputs: 5700bdccfc6209a5460dc124403eed6c3f5ba58da0123b392ab0b1fa23306f27:0
# selected output #1
# detecting unspent output as a transfer destination ...
# - no unspent outputs matching the address, a new transaction will be created
# - allocating the minimum amount of bitcoins taken from the automaatically selected unspent output
# - allocating the USDT change to an automatically created output
# asset transfer output created and saved into the file transfer_proof.dat

# $ kaleidoscope asset-add genesis.dat
# $ kaleidoscope asset-import history.dat
# $ kaleidoscope asset-export --for 02d1d80235fa5bba42e9612a7fe7cd74c6b2bf400c92d866f28d429846c679cceb
# >   bc1qwqdg6squsna38e46795at95yu9atm8azzmyvckulcc7kytlcckxswvvzej history.dat
```