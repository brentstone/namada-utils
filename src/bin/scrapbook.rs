use namada_sdk::address::{Address, ImplicitAddress};
use namada_sdk::eth_bridge::ethers::core::rand;
use namada_sdk::key::common::SecretKey;
use namada_sdk::key::ed25519::SigScheme as ed25519SigScheme;
use namada_sdk::key::{PublicKeyHash, RefTo, SigScheme};
use rand::rngs::OsRng;

#[tokio::main]
async fn main() {
    let mut rng = OsRng;
    let secret_key = ed25519SigScheme::generate(&mut rng);
    let secret_key = SecretKey::Ed25519(secret_key);

    let pkh = PublicKeyHash::from(&secret_key.ref_to());
    let address = Address::Implicit(ImplicitAddress(pkh));
    println!("Generated address: {:?}", address);
}
