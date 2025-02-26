use std::str::FromStr;

use namada_sdk::rpc;
use namada_token::Dec;
use namada_tools::{build_ctx, load_wallet};

#[tokio::main]
async fn main() {
    let (sdk, _config) = build_ctx().await;

    // Wallet things
    load_wallet(&sdk).await;

    let num_vals: usize = 25;

    let current_epoch = rpc::query_epoch(&sdk.client)
        .await
        .expect("Query epoch error");
    println!("Current epoch: {}\n", current_epoch);

    let mut consensus_validators = rpc::get_all_consensus_validators(&sdk.client, current_epoch)
        .await
        .expect("Query consensus validators error")
        .into_iter()
        .collect::<Vec<_>>();

    consensus_validators.sort_by(|a, b| b.bonded_stake.cmp(&a.bonded_stake));

    let total_stake = rpc::get_total_staked_tokens(&sdk.client, current_epoch)
        .await
        .unwrap();
    let total_stake = Dec::try_from(total_stake).unwrap();

    println!("Top {} validators by stake (with cumulative VP):", num_vals);
    let mut cumulative_stake_frac = Dec::zero();
    for (i, val) in consensus_validators.iter().enumerate() {
        if i >= num_vals {
            break;
        }
        let stake_frac = Dec::try_from(val.bonded_stake)
            .unwrap()
            .checked_div(total_stake)
            .unwrap();
        cumulative_stake_frac = cumulative_stake_frac.checked_add(stake_frac).unwrap();

        let val_metadata = rpc::query_metadata(&sdk.client, &val.address, Some(current_epoch))
            .await
            .unwrap()
            .0
            .unwrap();

        let name = if let Some(name) = val_metadata.name {
            name
        } else {
            String::from("None")
        };

        // let discord = if let Some(discord) = val_metadata.discord_handle {
        //     discord
        // } else {
        //     String::from("None")
        // };

        let stake_frac = stake_frac
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap();

        let cumulative_stake_frac = cumulative_stake_frac
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap();

        println!(
            "{}% ({}%) --- {}",
            &stake_frac.to_string()[..5],
            &cumulative_stake_frac.to_string()[..5],
            name //, discord,
                 // val_metadata.email
        );
    }
}
