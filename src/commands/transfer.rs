use super::balance::get_balance;
use crate::utils::{
    call_token_transfer_entry_point,
    constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL},
    format_with_thousands_separator, get_contract_cep18_hash_keys,
    keys::retrieve_private_key,
    prompt_yes_no,
};
use casper_rust_wasm_sdk::{
    helpers::motes_to_cspr,
    types::{key::Key, public_key::PublicKey},
};

pub async fn transfer(from: PublicKey, to: Key, amount: String) -> Option<String> {
    // Retrieve contract token hash and package hash
    let (cowl_cep18_token_contract_hash, _) = match get_contract_cep18_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => {
            log::error!("Failed to retrieve contract token hash and package hash.");
            return None;
        }
    };

    // Retrieve the private key
    let secret_key = retrieve_private_key(&from).await;

    let answer = prompt_yes_no(&format!(
        "Please confirm transfer of {} {} ({} {}) to {}?",
        format_with_thousands_separator(&motes_to_cspr(&amount).unwrap()),
        *COWL_CEP_18_TOKEN_SYMBOL,
        amount,
        *COWL_CEP_18_COOL_SYMBOL,
        &to.to_formatted_string()
    ));

    if !answer {
        log::warn!("Transfer aborted.");
        return None;
    }

    // Call the token transfer entry point
    call_token_transfer_entry_point(
        &cowl_cep18_token_contract_hash,
        &from,
        secret_key.expect("Failed to retrieve sender private key."),
        None, // owner is sender
        &to,
        amount,
    )
    .await;

    let to_balance = get_balance(None, Some(to)).await;
    Some(to_balance)
}

pub async fn print_transfer(from: PublicKey, to: Key, amount: String) {
    if let Some(balance) = transfer(from, to.clone(), amount).await {
        log::info!("Balance for {}", to.to_formatted_string());
        log::info!(
            "{} {}",
            format_with_thousands_separator(&motes_to_cspr(&balance).unwrap()),
            *COWL_CEP_18_TOKEN_SYMBOL
        );
        log::info!("{} {}", balance, *COWL_CEP_18_COOL_SYMBOL);
    }
}
