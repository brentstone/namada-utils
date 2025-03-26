use namada_core::token;
use namada_proof_of_stake::types::BondId;
use namada_sdk::{rpc, Namada};
use namada_utils::{build_ctx, get_addresses};

#[tokio::main]
async fn main() {
    let (sdk, config) = build_ctx().await;
    
    let wallet = sdk.wallet().await;
    let native_token = wallet.find_address("nam").expect("Native token address not found in wallet");

    let current_epoch = rpc::query_epoch(&sdk.client)
        .await
        .expect("Query epoch error");
    println!("Current epoch: {}\n", current_epoch);

    let my_addresses = get_addresses(&config);

    let mut total_balance = token::Amount::zero();
    let mut total_bonded = token::Amount::zero();
    let mut total_rewards = token::Amount::zero();
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

        let mut sources_rewards = token::Amount::zero();
        for (
            BondId {
                source: _,
                validator,
            },
            _,
        ) in bonds.data
        {
            let rewards = rpc::query_rewards(&sdk.client, &Some(addr.clone()), &validator, &None)
                .await
                .unwrap();
            // println!(
            //     "Rewards from validator {}: {} NAM",
            //     validator,
            //     rewards.to_string_native()
            // );
            sources_rewards = sources_rewards.checked_add(rewards).unwrap();
        }
        println!(
            "Unclaimed rewards: {} NAM",
            sources_rewards.to_string_native()
        );

        total_balance = total_balance.checked_add(balance).unwrap();
        total_bonded = total_bonded.checked_add(bonded).unwrap();
        total_rewards = total_rewards.checked_add(sources_rewards).unwrap();
        println!();
    }
    println!("Totals -------------------\n");
    println!("Total balance: {} NAM", total_balance.to_string_native());
    println!("Total bonded: {} NAM", total_bonded.to_string_native());
    println!(
        "Total unclaimed rewards: {} NAM",
        total_rewards.to_string_native()
    );
    let total_tokens = total_balance
        .checked_add(total_bonded)
        .unwrap()
        .checked_add(total_rewards)
        .unwrap();
    println!(
        "\nTotal transparent tokens to name: {} NAM",
        total_tokens.to_string_native()
    );
}
