use crate::utils::{
    call_withdraw_cspr_entry_point,
    config::get_key_pair_from_vesting,
    constants::{DEFAULT_BALANCE, INSTALLER},
    format_with_thousands_separator, get_contract_swap_hash_keys,
    keys::get_key_pair_from_key,
    prompt_yes_no,
};
use casper_rust_wasm_sdk::{helpers::motes_to_cspr, types::key::Key};
use std::process;

use super::balance::get_cspr_balance;

pub async fn withdraw_cspr(amount: String) -> Option<(String, String)> {
    let (_, cowl_swap_contract_package_hash) = match get_contract_swap_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => (String::from(""), String::from("")),
    };

    if cowl_swap_contract_package_hash.is_empty() {
        log::error!("Swap contract package does not exist in installer named keys");
        process::exit(1)
    }

    let key_pair = get_key_pair_from_vesting(INSTALLER).await.unwrap();

    let answer = prompt_yes_no(&format!(
        "Please confirm withdraw of {} {} ({} {})?",
        format_with_thousands_separator(&motes_to_cspr(&amount).unwrap()),
        "CSPR",
        amount,
        "motes",
    ));

    call_withdraw_cspr_entry_point(&key_pair, &cowl_swap_contract_package_hash, amount).await;

    if !answer {
        log::warn!("Withdraw aborted.");
        return None;
    }

    let key = Key::from_account(key_pair.public_key.to_account_hash());
    let (vesting_type, key_pair) = get_key_pair_from_key(&key).await;

    let default_balance = (DEFAULT_BALANCE.to_string(), DEFAULT_BALANCE.to_string());

    let identifier = key.to_formatted_string();

    let balance = match (vesting_type, key_pair) {
        (Some(vesting_type), Some(key_pair)) => get_cspr_balance(&key_pair, &vesting_type).await,
        (None, Some(key_pair)) => get_cspr_balance(&key_pair, &identifier).await,
        _ => default_balance,
    };
    Some(balance)
}

pub async fn print_withdraw_cspr(amount: String) {
    if let Some((balance, balance_motes)) = withdraw_cspr(amount).await {
        log::info!("Balance CSPR for Installer");
        log::info!("{} {}", format_with_thousands_separator(&balance), "CSPR");
        log::info!("{} {}", balance_motes, "motes");
    }
}
