use std::str::FromStr;

use clap::Parser;
use namada_sdk::{
    address::Address,
    args::{InputAmount, TxBuilder, TxTransparentSource, TxTransparentTarget},
    rpc,
    signing::default_sign,
    Namada,
};
use namada_utils::{build_ctx, load_keys, read_csv_to_vec};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Gas limit for the transaction
    #[arg(short, long = "gas-limit", default_value_t = 50_000)]
    gas_limit: u64,
    /// The token to transfer
    #[arg(short, long = "token")]
    token: String,
    /// The fee token to use for the transaction
    #[arg(
        short,
        long = "fee-token",
        default_value = "tnam1q9gr66cvu4hrzm0sd5kmlnjje82gs3xlfg3v6nu7"
    )]
    fee_token: String,
    /// The file of sources
    #[arg(
        long = "sources",
        default_value = "namada-utils/config/transfer_sources.csv"
    )]
    sources: String,
    /// The file of targets
    #[arg(
        long = "targets",
        default_value = "namada-utils/config/transfer_targets.csv"
    )]
    targets: String,
}

#[derive(Debug, Deserialize)]
struct TransferTarget {
    address: String,
    amount: u64,
}

fn load_transfer_targets(path: &str) -> Vec<TransferTarget> {
    read_csv_to_vec::<TransferTarget>(path).expect("Failed to read CSV")
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let gas_limit = args.gas_limit;
    let token = Address::from_str(&args.token).expect("Invalid token address");
    let fee_token = Address::from_str(&args.fee_token).expect("Invalid fee token address");

    let (sdk, _config) = build_ctx().await;

    // Wallet things
    load_keys(&sdk, "config/keys.csv").await;

    let transfer_sources = load_transfer_targets(&args.sources)
        .into_iter()
        .map(|a| TxTransparentSource {
            source: Address::from_str(&a.address).unwrap(),
            token: token.clone(),
            amount: InputAmount::from_str(a.amount.to_string().as_str()).unwrap(),
        })
        .collect::<Vec<_>>();
    let transfer_targets = load_transfer_targets(&args.targets)
        .into_iter()
        .map(|a| TxTransparentTarget {
            target: Address::from_str(&a.address).unwrap(),
            token: token.clone(),
            amount: InputAmount::from_str(a.amount.to_string().as_str()).unwrap(),
        })
        .collect::<Vec<_>>();

    // let native_token = sdk.wallet().await.find_address("nam").unwrap().into_owned();
    let key = sdk.wallet().await.find_public_key("key-0").unwrap();

    let mut transfer_tx_builder = sdk
        .new_transparent_transfer(transfer_sources, transfer_targets.clone())
        .signing_keys(vec![key.clone()])
        .gas_limit(gas_limit.into())
        .fee_token(fee_token.clone());

    let (mut transfer_tx, signing_data) = transfer_tx_builder
        .build(&sdk)
        .await
        .expect("unable to build transfer");

    sdk.sign(
        &mut transfer_tx,
        &transfer_tx_builder.tx,
        signing_data,
        default_sign,
        (),
    )
    .await
    .expect("unable to sign transparent-transfer tx");

    // Extract and display transaction hash in hex format
    let tx_hash = transfer_tx.header_hash();
    println!("Transaction Hash: {}", tx_hash);

    match sdk.submit(transfer_tx, &transfer_tx_builder.tx).await {
        Ok(res) => println!("\n\nTx result: {:?}\n\n", res),
        Err(e) => println!("\n\nTx error: {:?}\n\n", e),
    }

    // Print some results out
    for target in &transfer_targets {
        let target = &target.target;
        let balance = rpc::get_token_balance(&sdk.client, &token, target, None)
            .await
            .unwrap();
        println!("{}:  {}", &target, balance.to_string_native());
    }
}
