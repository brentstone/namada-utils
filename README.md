# namada-utils
Some tools and utilities for looking at Namada blockchain data

## Setup
1. Set the environment variable `$RPC_NAMADA_UTILS` to the RPC address of your choice on your system:
    ```
    export RPC_NAMADA_UTILS="<rpc-address>"
    ```
2. Install the `namada` binaries, particularly `namadac` and `namadaw`, and place them in your `$PATH`.
3. For any personal accounts for which you want to query information (like balances, bonds), place the addresses in `config/my_addresses.txt`. One address per line without commas.

## Installation

Simply run `cargo build`, then the binaries in `src/bin/` will be built and placed into `target/debug/`.