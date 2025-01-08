use crate::{
    commands::balance::get_balance,
    utils::{
        call_withdraw_cowl_entry_point,
        config::get_key_pair_from_vesting,
        constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL, INSTALLER},
        format_with_thousands_separator, get_contract_swap_hash_keys, prompt_yes_no,
    },
};
use casper_rust_wasm_sdk::{helpers::motes_to_cspr, types::key::Key};
use std::process;

pub async fn withdraw_cowl(amount: String) -> Option<String> {
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
        *COWL_CEP_18_TOKEN_SYMBOL,
        amount,
        *COWL_CEP_18_COOL_SYMBOL,
    ));

    call_withdraw_cowl_entry_point(&key_pair, &cowl_swap_contract_package_hash, amount).await;

    if !answer {
        log::warn!("Withdraw aborted.");
        return None;
    }

    let key = Some(Key::from_account(key_pair.public_key.to_account_hash()));

    let to_balance = get_balance(None, key).await;
    Some(to_balance)
}

pub async fn print_withdraw_cowl(amount: String) {
    if let Some(balance) = withdraw_cowl(amount).await {
        log::info!("Balance for Installer");
        log::info!(
            "{} {}",
            format_with_thousands_separator(&motes_to_cspr(&balance).unwrap()),
            *COWL_CEP_18_TOKEN_SYMBOL
        );
        log::info!("{} {}", balance, *COWL_CEP_18_COOL_SYMBOL);
    }
}
