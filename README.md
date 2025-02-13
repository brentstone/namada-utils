# namada-utils
Some tools and utilities for looking at Namada blockchain data

## Setup
1. Set some environment variables for the RPC of your choice and location of this repo's directory on your system:
    ```
    export RPC_NAMADA_UTILS="<rpc-address>"
    export NAMADA_UTILS_DIR="/path/to/repo"
    ```
2. Install the `namada` binaries, particularly `namadac` and `namadaw`, and place them in your `$PATH`. Currently compatible with [v1.1.1](https://github.com/anoma/namada/releases/tag/v1.1.1).
3. For any personal accounts for which you want to query information (like balances, bonds), place the addresses in `config/my_addresses.txt`. One address per line without commas.

## Installation

Simply run `cargo build`, then the binaries in `src/bin/` will be built and placed into `target/debug/`.
Binaries built in `--release` mode with be placed in `target/release/`.