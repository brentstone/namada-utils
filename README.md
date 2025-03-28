# Namada Utils

A collection of utilities and tools for interacting with and analyzing the Namada blockchain. This project provides various tools for querying blockchain data, performing batch operations, and managing transactions.

## Features

- Batch transfer operations
- Blockchain data querying and analysis
- Transaction management
- Integration with Namada SDK

## Prerequisites

- Rust toolchain (latest stable version recommended)
- Namada binaries (`namadac` and `namadaw`) v1.1.1 or compatible
- Access to a Namada RPC endpoint

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/namada-utils.git
   cd namada-utils
   ```

2. Build the project:
   ```bash
   cargo build
   ```
   Binaries will be available in:
   - Debug build: `target/debug/`
   - Release build: `target/release/` (recommended for production use)

## Configuration

1. Set up environment variables:
   ```bash
   export RPC_NAMADA_UTILS="<your-rpc-address>"
   export NAMADA_UTILS_DIR="/path/to/repo"
   ```

2. Configure your wallet:
   - Place your `wallet.toml` in `./sdk-wallet/wallet.toml`
   - This can be copied from your existing Namada wallet or created using the Namada SDK

3. Update the configuration:
   - Edit `config/config.toml` to include any transparent addresses you want to query
   - Add addresses with quotations or use aliases from your wallet

## Usage

### Batch Transfers

To perform batch transfers:

1. Configure source address:
   - Add your source address and private key to `config/keys.csv`

2. Set up transfer targets:
   - Add target addresses and amounts to `config/transfer_targets.csv`

3. Run the batch transfer:
   ```bash
   cargo run --bin batch_transfer
   ```

## Project Structure

```
namada-utils/
├── config/           # Configuration files
├── sdk-wallet/       # Wallet configuration
├── src/             # Source code
│   └── bin/         # Binary executables
├── Cargo.toml       # Project dependencies
└── README.md        # This file
```

## Dependencies

This project relies on the following main dependencies:
- namada_core
- namada_sdk
- namada_tx
- namada_governance
- namada_ibc
- namada_token
- namada_parameters
- namada_proof_of_stake

## License

This project is licensed under the terms specified in the LICENSE file.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.