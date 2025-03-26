use std::str::FromStr;

use namada_sdk::{
    address::Address,
    args::{InputAmount, TxBuilder, TxTransparentTransferData},
    io::NullIo,
    masp::fs::FsShieldedUtils,
    rpc,
    signing::default_sign,
    wallet::fs::FsWalletUtils,
    Namada, NamadaImpl,
};
use namada_utils::{build_ctx, load_keys, read_csv_to_vec};
use serde::Deserialize;
use tendermint_rpc::HttpClient;

#[derive(Debug, Deserialize)]
struct TransferTarget {
    address: String,
    amount: u64,
}

fn load_transfer_targets(path: &str) -> Vec<TransferTarget> {
    read_csv_to_vec::<TransferTarget>(path).expect("Failed to read CSV")
}

async fn build_transfer_data(
    sdk: &NamadaImpl<HttpClient, FsWalletUtils, FsShieldedUtils, NullIo>,
    source: &str,
    target: &str,
    token: &Address,
    raw_amount: u64,
) -> TxTransparentTransferData {
    let source = sdk
        .wallet()
        .await
        .find_address(source)
        .unwrap()
        .into_owned();
    let target = Address::from_str(target).unwrap();
    let amount = InputAmount::from_str(raw_amount.to_string().as_str()).unwrap();

    TxTransparentTransferData {
        source,
        target,
        token: token.clone(),
        amount,
    }
}

#[tokio::main]
async fn main() {
    // Parameters here for now
    // TODO: impl CLI args for these
    let gas_limit: u64 = 100_000;

    let (sdk, _config) = build_ctx().await;

    // Wallet things
    load_keys(&sdk, "config/keys.csv").await;

    let transfer_targets = load_transfer_targets("config/transfer_targets.csv");

    let native_token = sdk.wallet().await.find_address("nam").unwrap().into_owned();
    let key = sdk.wallet().await.find_public_key("key-0").unwrap();

    let token = native_token;

    let mut data = Vec::new();
    for target in &transfer_targets {
        data.push(build_transfer_data(&sdk, "key-0", &target.address, &token, target.amount).await);
    }

    let mut transfer_tx_builder = sdk
        .new_transparent_transfer(data)
        .signing_keys(vec![key.clone()])
        .gas_limit(gas_limit.into())
        .fee_token(token.clone());

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

    match sdk.submit(transfer_tx, &transfer_tx_builder.tx).await {
        Ok(res) => println!("Tx result: {:?}", res),
        Err(e) => println!("\n\nTx error: {:?}\n\n", e),
    }

    // Print some results out
    for target in transfer_targets {
        let target = Address::from_str(&target.address).unwrap();
        let balance = rpc::get_token_balance(&sdk.client, &token, &target, None)
            .await
            .unwrap();
        println!("{}:  {}", &target, balance.to_string_native());
    }
}
