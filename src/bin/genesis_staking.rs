use std::{fs::File, str::FromStr};

use csv::Writer;
use namada_core::token;
use namada_sdk::{
    address::Address,
    collections::HashMap,
    rpc::{self, enriched_bonds_and_unbonds},
};
use namada_token::Dec;
use namada_utils::{
    build_ctx, get_addresses_from_file, get_backer_balance, get_bonds_to_top_validators,
    get_core_balance, get_future_alloc_balance, get_genesis_accounts, get_pg_validator_balance,
    get_public_alloc_balance, get_rd_balance, Record,
};

#[tokio::main]
async fn main() {
    let (sdk, _config) = build_ctx().await;

    let current_epoch = rpc::query_epoch(&sdk.client)
        .await
        .expect("Query epoch error");
    println!("Current epoch: {}", current_epoch);

    let total_stake = rpc::get_total_staked_tokens(&sdk.client, current_epoch)
        .await
        .unwrap();
    println!("Total stake: {} NAM", total_stake.to_string_native());
    let total_staked_dec = Dec::try_from(total_stake).unwrap();

    println!("\n---------- Backers --------------------------\n");
    let mut backer_stake = token::Amount::zero();
    let mut backer_bonds_to_top_vals = HashMap::<String, token::Amount>::new();

    let file = File::create("output.csv").unwrap();
    let mut wtr = Writer::from_writer(file);
    wtr.write_record([
        "Address",
        "Total Stake",
        "Unit 410 [1]",
        "Unit 410 [2]",
        "Chorus One",
        "P2P",
        "Informal",
    ])
    .unwrap();

    let frac_of_validator =
        |val: &str, bonds: &HashMap<String, token::Amount>, tot_stake: token::Amount| -> Dec {
            if let Some(a) = bonds.get(val) {
                if tot_stake > token::Amount::zero() {
                    Dec::try_from(*a)
                        .unwrap()
                        .checked_div(Dec::try_from(tot_stake).unwrap())
                        .unwrap()
                } else {
                    Dec::zero()
                }
            } else {
                Dec::zero()
            }
        };

    let sources = get_addresses_from_file("config/backers.txt");
    for delegator in sources {
        let bonds =
            enriched_bonds_and_unbonds(&sdk.client, current_epoch, &Some(delegator.clone()), &None)
                .await
                .unwrap();

        let bonded = bonds.bonds_total_active().unwrap();
        backer_stake = backer_stake.checked_add(bonded).unwrap();

        let stake_to_top_val = get_bonds_to_top_validators(&bonds.data);
        for (name, bond_amt) in &stake_to_top_val {
            let b = backer_bonds_to_top_vals.entry(name.clone()).or_default();
            *b = b.checked_add(*bond_amt).unwrap();
        }
        let u410_frac1 = frac_of_validator("Unit 410 [1]", &stake_to_top_val, bonded);
        let u410_frac2 = frac_of_validator("Unit 410 [2]", &stake_to_top_val, bonded);
        let chorusone_frac = frac_of_validator("Chorus One", &stake_to_top_val, bonded);
        let p2p_frac = frac_of_validator("P2P.org", &stake_to_top_val, bonded);
        let informal_frac = frac_of_validator("Informal", &stake_to_top_val, bonded);

        wtr.write_record(&[
            delegator.to_string(),
            bonded.to_string_native(),
            u410_frac1.to_string(),
            u410_frac2.to_string(),
            chorusone_frac.to_string(),
            p2p_frac.to_string(),
            informal_frac.to_string(),
        ])
        .unwrap();
    }
    let backer_staked_dec = Dec::try_from(backer_stake).unwrap();
    let backer_balance_dec = Dec::try_from(get_backer_balance()).unwrap();
    let backer_frac = backer_staked_dec.checked_div(total_staked_dec).unwrap();
    println!(
        "Genesis balance: {} NAM\n",
        get_backer_balance().to_string_native()
    );
    println!(
        "Backer fraction of total stake: {}%\nFraction of backer tokens staked: {}%",
        backer_frac
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap(),
        backer_staked_dec
            .checked_div(backer_balance_dec)
            .unwrap()
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap()
    );

    let top_stake = backer_bonds_to_top_vals
        .iter()
        .fold(token::Amount::zero(), |acc, (_, x)| {
            acc.checked_add(*x).unwrap()
        });
    let top_stake_frac = Dec::try_from(top_stake)
        .unwrap()
        .checked_div(backer_staked_dec)
        .unwrap();

    println!(
        "\nFraction of backer stake held by top 5 validators: {}%",
        top_stake_frac
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap()
    );
    for (name, bonded) in backer_bonds_to_top_vals.iter() {
        let frac = Dec::try_from(*bonded)
            .unwrap()
            .checked_div(backer_staked_dec)
            .unwrap();
        println!(
            "  --> {}: {}%",
            name,
            frac.checked_mul(Dec::from_str("100").unwrap()).unwrap()
        );
    }

    wtr.flush().unwrap();
    println!("\nData written to output.csv\n");

    println!("\n---------- Core team --------------------------\n");
    let mut core_stake = token::Amount::zero();
    let mut core_team_bonds_to_top_vals = HashMap::<String, token::Amount>::new();

    for delegator in get_addresses_from_file("config/core_team.txt") {
        let bonds =
            enriched_bonds_and_unbonds(&sdk.client, current_epoch, &Some(delegator.clone()), &None)
                .await
                .unwrap();
        let bonded = bonds.bonds_total_active().unwrap();
        core_stake = core_stake.checked_add(bonded).unwrap();

        let stake_to_top_val = get_bonds_to_top_validators(&bonds.data);
        for (name, bond_amt) in stake_to_top_val.iter() {
            let b = core_team_bonds_to_top_vals.entry(name.clone()).or_default();
            *b = b.checked_add(*bond_amt).unwrap();
        }
    }
    let core_staked_dec = Dec::try_from(core_stake).unwrap();
    let core_balance_dec = Dec::try_from(get_core_balance()).unwrap();
    let core_frac = core_staked_dec.checked_div(total_staked_dec).unwrap();
    println!(
        "Genesis balance: {} NAM\n",
        get_core_balance().to_string_native()
    );
    println!(
        "Core team fraction of total stake: {}%\nFraction of core team tokens staked: {}%",
        core_frac
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap(),
        core_staked_dec
            .checked_div(core_balance_dec)
            .unwrap()
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap()
    );

    let top_stake = core_team_bonds_to_top_vals
        .iter()
        .fold(token::Amount::zero(), |acc, (_, x)| {
            acc.checked_add(*x).unwrap()
        });
    let top_stake_frac = Dec::try_from(top_stake)
        .unwrap()
        .checked_div(core_staked_dec)
        .unwrap();

    println!(
        "\nFraction of core team stake held by top 5 validators: {}%",
        top_stake_frac
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap()
    );

    println!("\n---------- R&D ecosystems -------------\n");
    let mut rd_stake = token::Amount::zero();
    let mut gen_balances = HashMap::<String, token::Amount>::new();
    let mut stakes = HashMap::<String, token::Amount>::new();

    for Record {
        address,
        amount,
        category: _,
        name,
    } in get_genesis_accounts("config/rd_ecosystem_dev.json")
    {
        let delegator = Address::from_str(&address).unwrap();
        let bonds =
            enriched_bonds_and_unbonds(&sdk.client, current_epoch, &Some(delegator.clone()), &None)
                .await
                .unwrap();
        let bonded = bonds.bonds_total_active().unwrap();
        rd_stake = rd_stake.checked_add(bonded).unwrap();

        let gen_balance = token::Amount::from(amount);
        let a = gen_balances.entry(name.clone()).or_default();
        *a = a.checked_add(gen_balance).unwrap();

        let b = stakes.entry(name).or_default();
        *b = b.checked_add(bonded).unwrap();
    }
    let rd_staked_dec = Dec::try_from(rd_stake).unwrap();
    let rd_balance_dec = Dec::try_from(get_rd_balance()).unwrap();
    let rd_frac = rd_staked_dec.checked_div(total_staked_dec).unwrap();
    println!(
        "Genesis balance: {} NAM",
        get_rd_balance().to_string_native()
    );
    for (name, balance) in gen_balances.iter() {
        println!("  --> {}: {}", name, balance.to_string_native(),);
    }

    println!(
        "\nR&D Ecosystem fraction of total stake: {}%\nFraction of R&D tokens staked: {}%",
        rd_frac.checked_mul(Dec::from_str("100").unwrap()).unwrap(),
        rd_staked_dec
            .checked_div(rd_balance_dec)
            .unwrap()
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap()
    );
    for (name, stake) in stakes.iter() {
        let balance = Dec::try_from(*gen_balances.get(name).unwrap()).unwrap();
        let stake_frac = Dec::try_from(*stake).unwrap().checked_div(balance).unwrap();
        let rd_frac = Dec::try_from(*stake)
            .unwrap()
            .checked_div(rd_staked_dec)
            .unwrap();
        println!(
            "  --> {}:\n            Fraction of balance staked: {}%\n            Fraction of total R&D stake: {}%",
            name,
            stake_frac
                .checked_mul(Dec::from_str("100").unwrap())
                .unwrap(),
            rd_frac.checked_mul(Dec::from_str("100").unwrap()).unwrap()
        );
    }

    println!("\n---------- Future allocations --------------------------\n");
    let mut future_alloc_stake = token::Amount::zero();
    let mut gen_balances = HashMap::<String, token::Amount>::new();
    for Record {
        address,
        amount,
        category: _,
        name,
    } in get_genesis_accounts("config/public_allocations_future.json")
    {
        let delegator = Address::from_str(&address).unwrap();
        let bonds =
            enriched_bonds_and_unbonds(&sdk.client, current_epoch, &Some(delegator.clone()), &None)
                .await
                .unwrap();
        let bonded = bonds.bonds_total_active().unwrap();
        future_alloc_stake = future_alloc_stake.checked_add(bonded).unwrap();

        let a = gen_balances.entry(name.clone()).or_default();
        *a = a.checked_add(token::Amount::from(amount)).unwrap();
    }

    let future_alloc_stake_dec = Dec::try_from(future_alloc_stake).unwrap();
    let future_alloc_frac = future_alloc_stake_dec
        .checked_div(total_staked_dec)
        .unwrap();
    println!(
        "Genesis balance: {} NAM",
        get_future_alloc_balance().to_string_native()
    );
    for (name, balance) in gen_balances.iter() {
        println!("  --> {}: {}", name, balance.to_string_native());
    }
    println!(
        "\nFuture allocations fraction of total stake: {}%",
        future_alloc_frac
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap()
    );

    println!("\n---------- Public allocations --------------------------\n");
    println!(
        "Genesis balance: {}",
        get_public_alloc_balance().to_string_native()
    );
    let rem_frac = Dec::one()
        .checked_sub(backer_frac)
        .unwrap()
        .checked_sub(core_frac)
        .unwrap()
        .checked_sub(rd_frac)
        .unwrap()
        .checked_sub(future_alloc_frac)
        .unwrap();
    println!(
        "\nAssumed public allocations fraction of total stake: {}%",
        rem_frac.checked_mul(Dec::from_str("100").unwrap()).unwrap()
    );

    let frac_pub_staked = rem_frac
        .checked_mul(total_staked_dec)
        .unwrap()
        .checked_div(Dec::try_from(get_public_alloc_balance()).unwrap())
        .unwrap();
    println!(
        "Fraction of public allocations staked: {}%",
        frac_pub_staked
            .checked_mul(Dec::from_str("100").unwrap())
            .unwrap()
    );

    let rem_tokens = token::Amount::native_whole(1_000_000_000)
        .checked_sub(get_backer_balance())
        .unwrap()
        .checked_sub(get_core_balance())
        .unwrap()
        .checked_sub(get_rd_balance())
        .unwrap()
        .checked_sub(get_future_alloc_balance())
        .unwrap()
        .checked_sub(get_public_alloc_balance())
        .unwrap();
    assert_eq!(rem_tokens, get_pg_validator_balance());
    println!();
}
