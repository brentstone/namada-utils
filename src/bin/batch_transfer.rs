use namada_sdk::{address::Address, Namada};
use namada_tools::{build_ctx, load_wallet};

struct TransferTarget {
    address: Address,
    amount: u64,
}

#[tokio::main]
async fn main() {
    let sdk = build_ctx().await;

    // Wallet things
    load_wallet(&sdk).await;

    let native_token = sdk.wallet().await.find_address("nam").unwrap().into_owned();
}
