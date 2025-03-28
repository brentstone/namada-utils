use std::str::FromStr;

use namada_proof_of_stake::rewards::PosRewardsRates;
use namada_sdk::{rpc, state::LastBlock, time::DateTimeUtc, Namada};
use namada_token::Dec;
use namada_utils::build_ctx;

fn convert_to_hours(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds - 3600 * hours) / 60;
    let seconds_unit = seconds - 3600 * hours - 60 * minutes;

    if hours > 0 {
        format!("{}h-{}m-{}s", hours, minutes, seconds_unit)
    } else if minutes > 0 {
        format!("{}m-{}s", minutes, seconds_unit)
    } else {
        format!("{}s", seconds_unit)
    }
}

#[tokio::main]
async fn main() {
    let (sdk, _config) = build_ctx().await;

    let native_token = sdk.wallet().await.find_address("nam").unwrap().into_owned();

    println!("\n---------- Block height and epoch -------------\n");
    match rpc::query_block(&sdk.client).await {
        Ok(last_block) => {
            let LastBlock { height, time } = last_block.unwrap();
            println!("Last block height: {} - (time: {})", height, time);
        }
        Err(e) => println!("Query error: {:?}", e),
    }

    let current_epoch = rpc::query_epoch(&sdk.client)
        .await
        .expect("Query epoch error");
    println!("Current epoch: {}\n", current_epoch);

    let (first_height_current_epoch, epoch_duration) = rpc::query_next_epoch_info(&sdk.client)
        .await
        .expect("Query next epoch info error");

    let first_header_current_epoch =
        rpc::query_block_header(&sdk.client, first_height_current_epoch)
            .await
            .unwrap()
            .unwrap();

    let first_block_time = first_header_current_epoch.time;
    let next_epoch_time = first_block_time + epoch_duration.min_duration;

    #[allow(clippy::disallowed_methods)]
    let current_time = DateTimeUtc::now();
    let seconds_left = next_epoch_time.time_diff(current_time).0;
    let time_remaining_str = convert_to_hours(seconds_left);

    println!(
        "First block height of epoch {current_epoch}: \
         {first_height_current_epoch}.\n"
    );
    println!(
        "Minimum number of blocks in an epoch: {}.",
        epoch_duration.min_num_of_blocks
    );
    println!(
        "Minimum amount of time for an epoch: {}.",
        convert_to_hours(epoch_duration.min_duration.0)
    );
    println!(
        "\nNext epoch ({}) begins in {} or at block height {}, whichever \
         occurs later.\n",
        current_epoch.next(),
        time_remaining_str,
        first_height_current_epoch.0 + epoch_duration.min_num_of_blocks + 2
    );

    println!("\n---------- Staking rewards -------------\n");
    match rpc::get_staking_rewards_rate(&sdk.client).await {
        Ok(PosRewardsRates {
            staking_rewards_rate,
            inflation_rate,
        }) => println!(
            "Annual staking rewards rate: {:?}%\nAnnual PoS inflation rate: {:?}%",
            staking_rewards_rate
                .checked_mul(Dec::from_str("100").unwrap())
                .unwrap(),
            inflation_rate
                .checked_mul(Dec::from_str("100").unwrap())
                .unwrap()
        ),
        Err(e) => println!("Query error: {:?}", e),
    }

    println!("\n---------- Balances -------------\n");
    let pgf_address = sdk.wallet().await.find_address("pgf").unwrap().into_owned();
    let gov_address = sdk.wallet().await.find_address("governance").unwrap().into_owned();
    let pgf_balance = rpc::get_token_balance(&sdk.client, &native_token, &pgf_address, None)
        .await
        .unwrap();
    let gov_balance = rpc::get_token_balance(&sdk.client, &native_token, &gov_address, None)
        .await
        .unwrap();
    println!("PGF balance: {} NAM", pgf_balance.to_string_native());
    println!("Gov balance: {} NAM", gov_balance.to_string_native());

    println!("\n---------- Staked tokens -------------\n");
    let total_staked_tokens = rpc::get_total_staked_tokens(&sdk.client, current_epoch)
        .await
        .unwrap();
    let native_supply = rpc::get_effective_native_supply(&sdk.client).await.unwrap();
    let staked_ratio = Dec::try_from(total_staked_tokens)
        .unwrap()
        .checked_div(Dec::try_from(native_supply).unwrap())
        .unwrap();

    println!(
        "Total bonded stake: {} NAM",
        total_staked_tokens.to_string_native()
    );
    println!(
        "Staked ratio: {}%",
        Dec::from_str("100")
            .unwrap()
            .checked_mul(staked_ratio)
            .unwrap()
    );

    let total_staked_tokens_pipeline =
        rpc::get_total_staked_tokens(&sdk.client, current_epoch.checked_add(2).unwrap())
            .await
            .unwrap();

    println!(
        "Total stake in 2 epochs: {}",
        total_staked_tokens_pipeline.to_string_native()
    );
}
