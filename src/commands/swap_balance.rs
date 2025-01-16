use crate::{commands::balance::print_balance, utils::get_contract_swap_hash_keys};
use casper_rust_wasm_sdk::types::key::Key;
use std::process;

pub async fn swap_balance() {
    let (_, cowl_swap_contract_package_hash) = match get_contract_swap_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => (String::from(""), String::from("")),
    };
    if cowl_swap_contract_package_hash.is_empty() {
        log::error!("Swap contract package does not exist in installer named keys");
        process::exit(1)
    }
    let key = Key::from_formatted_str(&cowl_swap_contract_package_hash).ok();
    log::info!(
        "Balance for Swap Contract Package {}",
        cowl_swap_contract_package_hash
    );
    print_balance(None, None, key).await;
}

pub async fn print_swap_balance() {
    swap_balance().await
}
