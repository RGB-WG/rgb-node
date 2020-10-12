Change Log
==========

v0.1.0-rc.2
-----------

### Main updates
- FFI and demo apps are moved into a separate 
  [**RGB SDK**](https://github.com/LNP-BP/rgb-sdk) project
- Big update and refactoring in RGB-20 achema (fungible assets)
  * Multiple inflation rights with better control over total inflation
  * Epoch-based burn and burn-and-replace procedures; enhanced with UTXO set and
    versioned proofs of burn data, supporting up to 15 burn proof variants 
    (+"no proofs" option)
  * Asset renomination procedure, for changing asset names or splitting stock 
    shares
  * Standardization of contract text URL and commitment format
  * Rights split procedure
  * Removed dust limit
- Proposal of RGB-21 schema after prolonged discussions (not available through
  API yet)
  * Unique tokens & token-specfic data
  * Issue control
  * Generic token data, internal or external, in different formats
  * Engravings (any why Schema subclassing is required)
  * LockUTXOs and descriptors for proof of reserves
  * Renominations
  * Rights splits
- New `rgb-cli` commands and `stash` daemon operations:
  * Listing available schemata
  * Exporting and inspecting schema in multiple formats (JSON, YAML, 
    StrictEncoding)
  * Listing known contracts
  * Exporting and inspecting contract genesis in multiple formats (JSON, YAML, 
    StrictEncoding)

### Breaking changes:
- Removal of dust limit parameter from command-line, RPC API calls and 
  FFI API integration points
