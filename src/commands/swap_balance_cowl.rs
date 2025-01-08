use crate::{
    commands::balance::get_balance,
    utils::{
        constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL},
        format_with_thousands_separator, get_contract_swap_hash_keys,
    },
};
use casper_rust_wasm_sdk::{helpers::motes_to_cspr, types::key::Key};
use std::process;

pub async fn swap_balance_cowl() -> Option<String> {
    let (_, cowl_swap_contract_package_hash) = match get_contract_swap_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => (String::from(""), String::from("")),
    };
    if cowl_swap_contract_package_hash.is_empty() {
        log::error!("Swap contract package does not exist in installer named keys");
        process::exit(1)
    }
    let key = Key::from_formatted_str(&cowl_swap_contract_package_hash).ok();
    let contract_balance = get_balance(None, key).await;
    Some(contract_balance)
}

pub async fn print_swap_balance_cowl() {
    if let Some(contract_balance) = swap_balance_cowl().await {
        log::info!("Balance for Swap contract");
        log::info!(
            "{} {}",
            format_with_thousands_separator(&motes_to_cspr(&contract_balance).unwrap()),
            *COWL_CEP_18_TOKEN_SYMBOL
        );
        log::info!("{} {}", contract_balance, *COWL_CEP_18_COOL_SYMBOL);
    }
}
