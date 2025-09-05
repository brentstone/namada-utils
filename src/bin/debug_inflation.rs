use std::str::FromStr;

use namada_core::token;
use namada_sdk::address::Address;
use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::collections::HashMap;
use namada_sdk::io::NullIo;
use namada_sdk::masp::fs::FsShieldedUtils;
use namada_sdk::rpc::{self, get_effective_native_supply, get_token_balance, query_storage_value};
use namada_sdk::wallet::fs::FsWalletUtils;
use namada_sdk::{Namada, NamadaImpl};
use namada_token::{Dec, Key};
use namada_utils::build_ctx;
use tendermint_rpc::HttpClient;

async fn get_masp_inflation_value<T>(
    sdk: &NamadaImpl<HttpClient, FsWalletUtils, FsShieldedUtils, NullIo>,
    token: &str,
    subkey: &str,
) -> T
where
    T: BorshDeserialize,
{
    let multitoken_addr = "tnam1pyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqej6juv";
    let key = format!("#{}/#{}/parameters/{}", multitoken_addr, token, subkey);
    let key = Key::from_str(&key).unwrap();
    let value = query_storage_value::<_, T>(&sdk.client, &key)
        .await
        .unwrap();
    value
}

#[tokio::main]
async fn main() {
    let (sdk, _config) = build_ctx().await;

    let tokens = HashMap::<_, _>::from_iter(vec![
        ("tnam1p5z5538v3kdk3wdx7r2hpqm4uq9926dz3ughcp7n", "stATOM"),
        ("tnam1ph6xhf0defk65hm7l5ursscwqdj8ehrcdv300u4g", "stTIA"),
        ("tnam1p4px8sw3am4qvetj7eu77gftm4fz4hcw2ulpldc7", "stOSMO"),
        ("tnam1p5z8ruwyu7ha8urhq2l0dhpk2f5dv3ts7uyf2n75", "OSMO"),
        ("tnam1pkg30gnt4q0zn7j00r6hms4ajrxn6f5ysyyl7w9m", "ATOM"),
        ("tnam1pklj3kwp0cpsdvv56584rsajty974527qsp8n0nm", "TIA"),
        ("tnam1pkl64du8p2d240my5umxm24qhrjsvh42ruc98f97", "USDC"),
    ]);

    let token_prices = HashMap::<_, _>::from_iter(vec![
        (
            "tnam1p5z5538v3kdk3wdx7r2hpqm4uq9926dz3ughcp7n",
            Dec::from_str("7.63").unwrap(),
        ),
        (
            "tnam1ph6xhf0defk65hm7l5ursscwqdj8ehrcdv300u4g",
            Dec::from_str("2.19").unwrap(),
        ),
        (
            "tnam1p4px8sw3am4qvetj7eu77gftm4fz4hcw2ulpldc7",
            Dec::from_str("0.259").unwrap(),
        ),
        (
            "tnam1p5z8ruwyu7ha8urhq2l0dhpk2f5dv3ts7uyf2n75",
            Dec::from_str("0.1865").unwrap(),
        ),
        (
            "tnam1pklj3kwp0cpsdvv56584rsajty974527qsp8n0nm",
            Dec::from_str("4.74").unwrap(),
        ),
        (
            "tnam1pkg30gnt4q0zn7j00r6hms4ajrxn6f5ysyyl7w9m",
            Dec::from_str("1.94").unwrap(),
        ),
        ("tnam1pkl64du8p2d240my5umxm24qhrjsvh42ruc98f97", Dec::one()),
    ]);

    let nam_supply = get_effective_native_supply(&sdk.client).await.unwrap();
    let nam_price = Dec::from_str("0.02").unwrap();

    for (token, token_name) in tokens {
        println!(
            "Token: {}\n------------------------------------",
            token_name
        );
        let last_inflation =
            get_masp_inflation_value::<token::Amount>(&sdk, token, "last_inflation").await;
        let last_locked_amount =
            get_masp_inflation_value::<token::Amount>(&sdk, token, "last_locked_amount").await;
        let locked_amount_target =
            get_masp_inflation_value::<token::Amount>(&sdk, token, "locked_amount_target").await;
        let max_reward_rate = get_masp_inflation_value::<Dec>(&sdk, token, "max_reward_rate").await;
        let proportional_gain =
            get_masp_inflation_value::<Dec>(&sdk, token, "proportional_gain").await;
        let derivative_gain = get_masp_inflation_value::<Dec>(&sdk, token, "derivative_gain").await;

        let percent_tgt = Dec::try_from(last_locked_amount)
            .unwrap()
            .checked_div(Dec::try_from(locked_amount_target).unwrap())
            .unwrap();
        let cur_inflation_rate = Dec::try_from(last_inflation)
            .unwrap()
            .checked_mul(365_u64)
            .unwrap()
            .checked_div(Dec::try_from(nam_supply).unwrap())
            .unwrap();
        let cur_nam_per_tok = Dec::try_from(last_inflation)
            .unwrap()
            .checked_div(Dec::try_from(last_locked_amount).unwrap())
            .unwrap();

        let cur_eff_apy = Dec::try_from(last_inflation)
            .unwrap()
            .checked_mul(365_u64)
            .unwrap()
            .checked_mul(nam_price)
            .unwrap()
            .checked_div(Dec::try_from(last_locked_amount).unwrap())
            .unwrap()
            .checked_div(*token_prices.get(token).unwrap())
            .unwrap();

        // let proportional_gain = Dec::from(100000_u64)
        //     .checked_mul(proportional_gain)
        //     .unwrap();
        // let derivative_gain = Dec::from(100000_u64).checked_mul(derivative_gain).unwrap();

        println!("Last inflation: {}", last_inflation);
        println!("Last locked amount: {}", last_locked_amount);
        println!("Locked amount target: {}", locked_amount_target);
        println!(
            "Max reward rate: {}%",
            max_reward_rate.checked_mul(Dec::from(100_u64)).unwrap()
        );
        println!("P gain: {}", proportional_gain);
        println!("D gain: {}", derivative_gain);
        println!("------------------------------------");
        println!("Using the last inflation amount to compute");
        println!("------------------------------------");
        println!(
            "Percent target: {}%",
            percent_tgt.checked_mul(100_u64).unwrap()
        );
        println!(
            "Current inflation rate: {}%",
            cur_inflation_rate.checked_mul(Dec::from(100_u64)).unwrap()
        );
        println!(
            "Current effective APY: {}%",
            cur_eff_apy.checked_mul(Dec::from(100_u64)).unwrap()
        );
        println!("Current NAM per token: {}", cur_nam_per_tok);

        // Run inflation mech early
        let controller = namada_controller::PDController::new(
            nam_supply.raw_amount(),
            max_reward_rate,
            last_inflation.raw_amount(),
            proportional_gain,
            derivative_gain,
            365_u64,
            Dec::try_from(locked_amount_target.raw_amount()).unwrap(),
            Dec::try_from(last_locked_amount.raw_amount()).unwrap(),
        );

        let token_addr = Address::from_str(token).unwrap();
        let masp_address = sdk
            .wallet()
            .await
            .find_address("masp")
            .unwrap()
            .into_owned();
        let cur_locked_amount = get_token_balance(&sdk.client, &token_addr, &masp_address, None)
            .await
            .unwrap();
        let control_coeff = max_reward_rate.checked_div(365_u64).unwrap();

        let inflation = controller
            .compute_inflation(
                control_coeff,
                Dec::try_from(cur_locked_amount.raw_amount()).unwrap(),
            )
            .unwrap();
        let inflation_rate = Dec::try_from(inflation)
            .unwrap()
            .checked_mul(365_u64)
            .unwrap()
            .checked_div(Dec::try_from(nam_supply).unwrap())
            .unwrap()
            .checked_div(Dec::from(1000000_u64))
            .unwrap();
        let cur_token_amount =
            rpc::get_token_balance(&sdk.client, &token_addr, &masp_address, None)
                .await
                .unwrap();
        let cur_nam_per_tok = Dec::try_from(inflation)
            .unwrap()
            .checked_div(Dec::try_from(cur_token_amount.raw_amount()).unwrap())
            .unwrap();

        let cur_eff_apy = Dec::try_from(last_inflation)
            .unwrap()
            .checked_mul(365_u64)
            .unwrap()
            .checked_mul(nam_price)
            .unwrap()
            .checked_div(Dec::try_from(last_locked_amount).unwrap())
            .unwrap()
            .checked_div(*token_prices.get(token).unwrap())
            .unwrap();

        println!("------------------------------------");
        println!(
            "Using the current amount in the MASP to run the PD Controller if epoch ended now"
        );
        println!("------------------------------------");
        println!("Exp. Inflation now: {}", inflation);
        println!("Current token amount: {}", cur_token_amount);
        println!(
            "Inflation rate now: {}%",
            inflation_rate.checked_mul(Dec::from(100_u64)).unwrap()
        );
        println!("Current NAM per token: {}", cur_nam_per_tok);
        println!(
            "Current effective APY: {}%",
            cur_eff_apy.checked_mul(Dec::from(100_u64)).unwrap()
        );
        println!(
            "Now / last = {}",
            Dec::try_from(inflation)
                .unwrap()
                .checked_div(Dec::try_from(last_inflation.raw_amount()).unwrap())
                .unwrap()
        );
        println!("------------------------------------\n");
    }
}
