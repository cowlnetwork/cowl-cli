use crate::utils::{
    call_token_set_allowance_entry_point,
    constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL},
    format_with_thousands_separator, get_contract_cep18_hash_keys, get_dictionary_item_params,
    keys::retrieve_private_key,
    prompt_yes_no, sdk, stored_value_to_parsed_string,
};
use casper_rust_wasm_sdk::{
    helpers::{make_dictionary_item_key, motes_to_cspr},
    types::{key::Key, public_key::PublicKey},
};
use cowl_vesting::constants::DICT_ALLOWANCES;
use serde_json::to_string;

pub async fn get_allowance(owner: &Key, spender: &Key) -> String {
    // Retrieve contract token hash and package hash
    let (cowl_cep18_token_contract_hash, _) = match get_contract_cep18_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => {
            log::error!("Failed to retrieve contract token hash and package hash.");
            return 0.to_string();
        }
    };

    let dictionary_key = make_dictionary_item_key(owner, spender);

    // Get the dictionary item parameters for the allowance
    let dictionary_item = get_dictionary_item_params(
        &cowl_cep18_token_contract_hash.to_string(),
        DICT_ALLOWANCES,
        &dictionary_key,
    );

    // Query the contract dictionary for the allowance
    let allowance_result = sdk()
        .query_contract_dict(dictionary_item, None::<&str>, None, None)
        .await;

    // Handle query result and extract stored value
    let stored_value = match allowance_result {
        Ok(result) => result.result.stored_value,
        Err(err) => {
            log::error!("Failed to query allowance from the contract.{}", err);
            return 0.to_string();
        }
    };

    let json_string = match to_string(&stored_value) {
        Ok(s) => s,
        Err(_) => {
            log::error!("Failed to serialize stored value into JSON.");
            return 0.to_string();
        }
    };
    stored_value_to_parsed_string(&json_string).unwrap_or_default()
}

pub async fn print_get_allowance(owner: &Key, spender: &Key) {
    let allowance = get_allowance(owner, spender).await;

    log::info!("Allowance for {}", spender.to_formatted_string());
    log::info!("{} {}", allowance, *COWL_CEP_18_COOL_SYMBOL);
    log::info!(
        "{} {}",
        format_with_thousands_separator(&motes_to_cspr(&allowance).unwrap()),
        *COWL_CEP_18_TOKEN_SYMBOL
    );
}

pub async fn set_allowance(
    owner: &PublicKey,
    spender: &Key,
    amount: String,
    decrease: bool,
) -> Option<String> {
    // Retrieve contract token hash and package hash
    let (cowl_cep18_token_contract_hash, _) = match get_contract_cep18_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => {
            log::error!("Failed to retrieve contract token hash and package hash.");
            return None;
        }
    };

    // Retrieve the private key
    let secret_key = retrieve_private_key(owner).await;

    let answer = prompt_yes_no(&format!(
        "Please confirm {} allowance of {} () {} for {}?",
        if decrease { "decreasing" } else { "increasing" },
        format_with_thousands_separator(&motes_to_cspr(&amount).unwrap()),
        *COWL_CEP_18_TOKEN_SYMBOL,
        &spender.to_formatted_string()
    ));

    if !answer {
        log::warn!("Setting allowance aborted.");
        return None;
    }

    // Call the token decrease/increase entry point
    call_token_set_allowance_entry_point(
        &cowl_cep18_token_contract_hash,
        owner,
        secret_key.expect("Failed to retrieve sender private key."),
        spender,
        amount,
        decrease,
    )
    .await;

    let to_allowance = get_allowance(&Key::from_account(owner.to_account_hash()), spender).await;
    Some(to_allowance)
}

pub async fn print_increase_allowance(owner: &PublicKey, spender: &Key, amount: String) {
    if let Some(allowance) = set_allowance(owner, spender, amount, false).await {
        log::info!("Increase allowance for {}", spender.to_formatted_string());
        log::info!(
            "{} {}",
            format_with_thousands_separator(&motes_to_cspr(&allowance).unwrap()),
            *COWL_CEP_18_TOKEN_SYMBOL
        );
    }
}

pub async fn print_decrease_allowance(owner: &PublicKey, spender: &Key, amount: String) {
    if let Some(allowance) = set_allowance(owner, spender, amount, false).await {
        log::info!("Decrease allowance for {}", spender.to_formatted_string());
        log::info!(
            "{} {}",
            format_with_thousands_separator(&motes_to_cspr(&allowance).unwrap()),
            *COWL_CEP_18_TOKEN_SYMBOL
        );
    }
}
