use crate::{
    commands::balance::print_balance,
    utils::{
        call_withdraw_cspr_entry_point, config::get_key_pair_from_vesting, constants::INSTALLER,
        format_with_thousands_separator, get_contract_swap_hash_keys, prompt_yes_no,
    },
};
use casper_rust_wasm_sdk::{helpers::motes_to_cspr, types::key::Key};
use std::process;

pub async fn withdraw_cspr(amount: String) {
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
        return;
    }

    let key = Key::from_account(key_pair.public_key.to_account_hash());
    log::info!("Balance for {}", key_pair.public_key.to_string());
    print_balance(None, Some(key.clone()), None).await;

    let key = Key::from_formatted_str(&cowl_swap_contract_package_hash).ok();
    log::info!(
        "Balance for Swap Contract Package {}",
        cowl_swap_contract_package_hash
    );
    print_balance(None, None, key).await;
}

pub async fn print_withdraw_cspr(amount: String) {
    withdraw_cspr(amount).await
}
