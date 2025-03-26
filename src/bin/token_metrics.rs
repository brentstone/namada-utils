use namada_sdk::{collections::HashMap, rpc, Namada};
use namada_utils::{build_ctx, get_address_from_ibc_denom};

pub fn get_mainnet_ibc_nicknames() -> HashMap<String, String> {
    HashMap::from_iter(vec![
        (
            String::from("transfer/channel-1/uosmo"),
            String::from("OSMO"),
        ),
        (
            String::from("transfer/channel-2/uatom"),
            String::from("ATOM"),
        ),
        (String::from("transfer/channel-3/utia"), String::from("TIA")),
        (
            String::from("transfer/channel-0/stuosmo"),
            String::from("stOSMO"),
        ),
        (
            String::from("transfer/channel-0/stuatom"),
            String::from("stATOM"),
        ),
        (
            String::from("transfer/channel-0/stutia"),
            String::from("stTIA"),
        ),
    ])
}

#[tokio::main]
async fn main() {
    let (sdk, config) = build_ctx().await;

    // let native_token = sdk.wallet().await.find_address("nam").unwrap().into_owned();
    let masp_address = sdk
        .wallet()
        .await
        .find_address("masp")
        .unwrap()
        .into_owned();

    let ibc_tokens = config.ibc_tokens;
    let ibc_nicknames = get_mainnet_ibc_nicknames();

    println!("\n--- Non-native tokens in Config --------");
    for trace in &ibc_tokens {
        if let Some(token) = ibc_nicknames.get(trace) {
            println!("{}: {}", token, trace);
        } else {
            println!("{trace}");
        }
    }

    println!("\n--- Total supply in Namada --------");
    for denom in &ibc_tokens {
        let token = ibc_nicknames.get(denom).unwrap_or(denom);
        let total_supply =
            rpc::get_token_total_supply(&sdk.client, &get_address_from_ibc_denom(denom))
                .await
                .expect("Query total supply error");
        println!("{}: {}", token, total_supply.to_string_native());
    }

    println!("\n--- Total supply in the MASP --------");
    for denom in &ibc_tokens {
        let token = ibc_nicknames.get(denom).unwrap_or(denom);
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
