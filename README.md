# namada-utils
Some tools and utilities for looking at Namada blockchain data

## Setup
1. Set some environment variables for the RPC of your choice and location of this repo's directory on your system:
    ```
    export RPC_NAMADA_UTILS="<rpc-address>"
    export NAMADA_UTILS_DIR="/path/to/repo"
    ```
2. Install the `namada` binaries, particularly `namadac` and `namadaw`, and place them in your `$PATH`. Currently compatible with [v1.1.1](https://github.com/anoma/namada/releases/tag/v1.1.1).
3. Fill in the `config/config.toml`. Any transparent addresses, like your own, that you would like to query in various programs can be placed in there, with quotations around the address.

## Batch transfers
Transparent batch transfers from one address to an arbitrary number of addresses each with arbitrary amounts are currently supported with the `batch_transfer` binary. Provide the following to successfully run the binary:
- the source address and private key into `config/keys.csv`
- each target address and amount placed into `config/transfer_targets.csv`

## Installation

Simply run `cargo build`, then the binaries in `src/bin/` will be built and placed into `target/debug/`.
Binaries built in `--release` mode with be placed in `target/release/`.