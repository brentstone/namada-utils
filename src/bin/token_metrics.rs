use namada_sdk::{rpc, Namada};
use namada_tools::{build_ctx, get_address_from_ibc_denom, get_ibc_tokens, load_wallet};

#[tokio::main]
async fn main() {
    let sdk = build_ctx().await;

    // Wallet things
    load_wallet(&sdk).await;

    // let native_token = sdk.wallet().await.find_address("nam").unwrap().into_owned();
    let masp_address = sdk
        .wallet()
        .await
        .find_address("masp")
        .unwrap()
        .into_owned();

    let ibc_tokens = get_ibc_tokens();

    println!("\n--- Whitelisted non-native tokens --------");
    for (token, denom) in &ibc_tokens {
        println!("{token}: {denom}");
    }

    println!("\n--- Total supply in Namada --------");
    for (token, denom) in &ibc_tokens {
        let total_supply =
            rpc::get_token_total_supply(&sdk.client, &get_address_from_ibc_denom(denom))
                .await
                .expect("Query total supply error");
        println!("{}: {}", token, total_supply.to_string_native());
    }

    println!("\n--- Total supply in the MASP --------");
    for (token, denom) in &ibc_tokens {
        let masp_balance = rpc::get_token_balance(
            &sdk.client,
            &get_address_from_ibc_denom(denom),
            &masp_address,
            None,
        )
        .await
        .expect("Query total supply error");
        println!("{}: {}", token, masp_balance.to_string_native());
    }
}
