use namada_core::token;
use namada_sdk::{rpc, Namada};
use namada_tools::{build_ctx, get_addresses, load_wallet};

#[tokio::main]
async fn main() {
    let sdk = build_ctx().await;

    // Wallet things
    load_wallet(&sdk).await;

    let native_token = sdk.wallet().await.find_address("nam").unwrap().into_owned();

    let current_epoch = rpc::query_epoch(&sdk.client)
        .await
        .expect("Query epoch error");
    println!("Current epoch: {}\n", current_epoch);

    let my_addresses = get_addresses("./config/my_addresses.txt");

    let mut total_balance = token::Amount::zero();
    let mut total_bonded = token::Amount::zero();
    for (i, addr) in my_addresses.iter().enumerate() {
        println!("Address-{i}:");
        let balance = rpc::get_token_balance(&sdk.client, &native_token, addr, None)
            .await
            .unwrap();
        println!("Balance: {} NAM", balance.to_string_native());
        // let rewards = rpc::rewa
        let bonds =
            rpc::enriched_bonds_and_unbonds(&sdk.client, current_epoch, &Some(addr.clone()), &None)
                .await
                .unwrap();
        let bonded = bonds.bonds_total_active().unwrap();
        println!("Bonded: {} NAM", bonded.to_string_native());

        total_balance = total_balance.checked_add(balance).unwrap();
        total_bonded = total_bonded.checked_add(bonded).unwrap();
        println!();
    }
    println!("Total balance: {} NAM", total_balance.to_string_native());
    println!("Total bonded: {} NAM", total_bonded.to_string_native());
    println!(
        "Total tokens: {} NAM",
        total_balance
            .checked_add(total_bonded)
            .unwrap()
            .to_string_native()
    );
}
