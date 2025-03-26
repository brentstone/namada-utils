use namada_core::token;
use namada_ibc::trace::ibc_token;
use namada_proof_of_stake::types::{BondId, BondsAndUnbondsDetail};
use namada_sdk::address::Address;
use namada_sdk::collections::HashMap;
use namada_sdk::key::common::SecretKey;
use namada_sdk::queries::vp::pos::Enriched;
use namada_sdk::wallet::Wallet;
use namada_sdk::{
    args::TxBuilder,
    chain::ChainId,
    io::NullIo,
    masp::{fs::FsShieldedUtils, ShieldedContext},
    wallet::fs::FsWalletUtils,
    Namada, NamadaImpl,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::from_reader;
use std::error::Error;
use std::fmt::Debug;
use std::io::BufReader;
use std::str::FromStr;
use namada_sdk::tendermint_rpc::{HttpClient, Url};

pub const RPC_ENV_VAR: &str = "RPC_NAMADA_UTILS";
pub const NAMADA_UTILS_DIR: &str = "NAMADA_UTILS_DIR";

// Genesis balances
pub fn get_backer_balance() -> token::Amount {
    token::Amount::native_whole(320_364_605)
}

pub fn get_rd_balance() -> token::Amount {
    token::Amount::native_whole(170_000_000)
}

pub fn get_core_balance() -> token::Amount {
    token::Amount::from(187986994166096)
}

pub fn get_future_alloc_balance() -> token::Amount {
    token::Amount::from(160539918535390)
}

pub fn get_public_alloc_balance() -> token::Amount {
    token::Amount::from(161108277298514)
}

pub fn get_pg_validator_balance() -> token::Amount {
    token::Amount::native_whole(205)
}

fn get_full_path(rel_path: &str) -> String {
    let base_dir = std::env::var(NAMADA_UTILS_DIR).expect("NAMADA_UTILS_DIR env var not set");
    format!("{base_dir}/{rel_path}")
}

#[derive(Debug, Deserialize)]
pub struct ConfigParams {
    pub chain_id: String,
    pub ibc_tokens: Vec<String>,
    pub transparent_addresses: Vec<String>,
}

fn load_config(rel_path: &str) -> Result<ConfigParams, Box<dyn std::error::Error>> {
    let path = get_full_path(rel_path);
    let contents = std::fs::read_to_string(path.as_str())?;
    let config: ConfigParams = toml::from_str(&contents)?;
    Ok(config)
}

pub async fn build_ctx() -> (
    NamadaImpl<HttpClient, FsWalletUtils, FsShieldedUtils, NullIo>,
    ConfigParams,
) {
    let rpc_url = std::env::var(RPC_ENV_VAR).expect("RPC_NAMADA_UTILS env var not set");
    let url = Url::from_str(&rpc_url).expect("Invalid RPC address");
    let http_client = HttpClient::new(url).unwrap();

    let mut wallet = FsWalletUtils::new("./sdk-wallet".into());
    wallet.load().expect("Failed to load wallet");

    let shielded_ctx = ShieldedContext::new(FsShieldedUtils::new("./masp".into()));
    let null_io = NullIo;

    let config = load_config("config/config.toml").expect("Could not load config file");

    let sdk = NamadaImpl::new(http_client, wallet, shielded_ctx.into(), null_io)
        .await
        .expect("unable to initialize Namada context")
        .chain_id(ChainId::from_str(&config.chain_id).unwrap());

    (sdk, config)
}

pub fn get_addresses(wallet: &Wallet<FsWalletUtils>, config: &ConfigParams) -> Vec<Address> {
    config
        .transparent_addresses
        .iter()
        .map(|addr| if addr.starts_with("tnam") {Address::from_str(addr).expect("Could not parse address")} else {
            wallet.find_address(addr).expect("Could not find address in wallet").into_owned()
        })
        .collect()
}

pub fn get_addresses_from_file(rel_path: &str) -> Vec<Address> {
    let path = get_full_path(rel_path);
    let addresses = std::fs::read_to_string(path).expect("Could not read addresses file");
    addresses
        .lines()
        .map(|line| Address::from_str(line).expect("Could not parse address"))
        .collect()
}

pub fn get_address_from_ibc_denom(denom: &str) -> Address {
    ibc_token(denom)
}

#[derive(Deserialize, Debug)]
pub struct Record {
    pub address: String,
    pub amount: u64,
    pub category: String,
    pub name: String,
}

pub fn get_genesis_accounts(rel_path: &str) -> Vec<Record> {
    let path = get_full_path(rel_path);
    let file = std::fs::File::open(path).expect("Could not open genesis accounts file");
    let reader = BufReader::new(file);
    from_reader(reader).expect("Could not parse genesis accounts file")
}

pub fn get_top_validators() -> HashMap<String, Address> {
    let top_vals = [
        (
            "Unit 410 [1]",
            "tnam1qyctcwkgthr06k7lx38zmjka5dakmvhhyyr0zafu",
        ),
        (
            "Unit 410 [2]",
            "tnam1q9vnysn3jj9l3rnucr0zt4jpuy224wdl7c0gezrj",
        ),
        (
            "Chorus One",
            "tnam1qxsx2ezu89gx252kwwluqp7hadyp285tkczhaqg0",
        ),
        ("P2P.org", "tnam1q8jrrf8s22cwd22yxhwc38tlvahplh2wyqjzl9gx"),
        ("Informal", "tnam1q9vrp45qtphed4q2vc382qrtf2gfykf50vssfe2h"),
    ];
    top_vals
        .iter()
        .map(|(name, addr)| (name.to_string(), Address::from_str(addr).unwrap()))
        .collect()
}

pub fn get_bonds_to_top_validators(
    bonds: &HashMap<BondId, Enriched<BondsAndUnbondsDetail>>,
) -> HashMap<String, token::Amount> {
    let mut bonds_to_top_validators = HashMap::<String, token::Amount>::new();
    if bonds.is_empty() {
        return bonds_to_top_validators;
    }

    let source = bonds.first().unwrap().0.clone().source;
    for (name, address) in get_top_validators() {
        let bond_id = BondId {
            source: source.clone(),
            validator: address.clone(),
        };
        if let Some(bonds) = bonds.get(&bond_id) {
            // println!("YES");
            let bonded = bonds.bonds_total_active().unwrap();
            // dbg!(&name, &bonded);
            let a = bonds_to_top_validators.entry(name).or_default();
            *a = a.checked_add(bonded).unwrap();
        } else {
            bonds_to_top_validators.insert(name, token::Amount::zero());
        }
    }
    bonds_to_top_validators
}

// Function to read a CSV file and parse it into an object that can be implemented later
pub fn read_csv_to_vec<T>(rel_path: &str) -> Result<Vec<T>, Box<dyn Error>>
where
    T: Debug + DeserializeOwned,
{
    let path = get_full_path(rel_path);
    let file = std::fs::File::open(path).expect("Could not open genesis accounts file");
    let mut rdr = csv::Reader::from_reader(file);

    let mut entries = Vec::new();
    for result in rdr.deserialize() {
        let record: T = result?;
        entries.push(record);
    }

    Ok(entries)
}

#[derive(Debug, Deserialize)]
struct AddressWithKey {
    address: String,
    pk: String,
}

pub async fn load_keys(
    sdk: &NamadaImpl<HttpClient, FsWalletUtils, FsShieldedUtils, NullIo>,
    rel_path: &str,
) {
    let data = read_csv_to_vec::<AddressWithKey>(rel_path).expect("Failed to read CSV");
    for (idx, AddressWithKey { address, pk }) in data.iter().enumerate() {
        println!("{}: {}", address, pk);
        let sk = SecretKey::from_str(pk).expect("Failed to parse secret key");
        let addr = Address::from_str(address).expect("Failed to parse address");

        sdk.wallet_mut()
            .await
            .insert_keypair(format!("key-{}", idx), false, sk, None, Some(addr), None)
            .expect("Failed to store keypair in wallet");
    }
    sdk.wallet().await.save().expect("Could not save wallet!");
}

// Write some tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_balance() {
        let sum = get_backer_balance()
            .checked_add(get_rd_balance())
            .unwrap()
            .checked_add(get_core_balance())
            .unwrap()
            .checked_add(get_future_alloc_balance())
            .unwrap()
            .checked_add(get_public_alloc_balance())
            .unwrap()
            .checked_add(get_pg_validator_balance())
            .unwrap();
        assert_eq!(sum, token::Amount::native_whole(1_000_000_000));
    }
}
