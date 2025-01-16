use crate::utils::{
    config::get_key_pair_from_vesting,
    constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL, DEFAULT_BALANCE},
    format_with_thousands_separator, get_contract_cep18_hash_keys, get_contract_swap_purse,
    get_dictionary_item_params,
    keys::{get_key_pair_from_key, KeyPair},
    sdk, stored_value_to_parsed_string,
};
use casper_rust_wasm_sdk::{
    helpers::{get_base64_key_from_account_hash, get_base64_key_from_key_hash, motes_to_cspr},
    types::{key::Key, purse_identifier::PurseIdentifier},
};
use cowl_vesting::{constants::DICT_BALANCES, enums::VestingType};
use indexmap::IndexMap;
use serde_json::to_string;

pub async fn get_balance(
    maybe_vesting_type: Option<VestingType>,
    maybe_key: Option<Key>,
) -> String {
    let dictionary_key = if let Some(vesting_type) = maybe_vesting_type {
        let key_pair = get_key_pair_from_vesting(&vesting_type.to_string())
            .await
            .unwrap();
        get_base64_key_from_account_hash(
            &key_pair.public_key.to_account_hash().to_formatted_string(),
        )
        .unwrap_or_else(|err| {
            log::error!(
                "Failed to retrieve account_hash for {}: {:?}",
                vesting_type,
                err
            );
            "".to_string()
        })
    } else if let Some(key) = maybe_key.clone() {
        if key.to_formatted_string().contains("account") {
            get_base64_key_from_account_hash(
                &key.clone()
                    .into_account()
                    .expect("get_balance method expects an account")
                    .to_formatted_string(),
            )
            .unwrap_or_else(|err| {
                log::error!(
                    "Failed to retrieve account hash for {}: {:?}",
                    key.to_formatted_string(),
                    err
                );
                "".to_string()
            })
        } else {
            get_base64_key_from_key_hash(&key.clone().to_formatted_string()).unwrap_or_else(|err| {
                log::error!(
                    "Failed to retrieve contract hash for {}: {:?}",
                    key.to_formatted_string(),
                    err
                );
                "".to_string()
            })
        }
    } else {
        log::error!("Both vesting_type and vesting_key are missing.");
        return DEFAULT_BALANCE.to_string();
    };

    // Retrieve contract token hash and package hash
    let (cowl_cep18_token_contract_hash, _) = match get_contract_cep18_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => {
            log::error!("Failed to retrieve contract token hash and package hash.");
            return DEFAULT_BALANCE.to_string();
        }
    };

    // Get the dictionary item parameters for the balance
    let dictionary_item = get_dictionary_item_params(
        &cowl_cep18_token_contract_hash.to_string(),
        DICT_BALANCES,
        &dictionary_key,
    );

    // Query the contract dictionary for the balance
    let balance_result = sdk()
        .query_contract_dict(dictionary_item, None::<&str>, None, None)
        .await;

    // Handle query result and extract stored value
    let stored_value = match balance_result {
        Ok(result) => result.result.stored_value,
        Err(err) => {
            if let Some(vesting_type) = maybe_vesting_type {
                log::warn!(
                    "No {} balance for {}!\n- Account Hash: {}",
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    vesting_type,
                    maybe_key
                        .as_ref()
                        .map(|key| key.to_formatted_string())
                        .unwrap_or_else(|| "Unknown Key".to_string())
                );
            } else if let Some(key) = maybe_key {
                let (vesting_type, key_pair) = get_key_pair_from_key(&key).await;
                if let Some(key_pair) = key_pair {
                    log::warn!(
                        "No {} balance for {}\n\
                        - Private Key: {:?}\n\
                        - Public Key: {}\n\
                        - Account Hash: {}",
                        *COWL_CEP_18_TOKEN_SYMBOL,
                        vesting_type.unwrap_or_default(),
                        key_pair.private_key_base64,
                        key_pair.public_key.to_string(),
                        key_pair.public_key.to_account_hash().to_formatted_string()
                    );
                } else {
                    log::warn!(
                        "No {} balance!\n- Account Hash: {}",
                        *COWL_CEP_18_TOKEN_SYMBOL,
                        key.to_formatted_string()
                    );
                }
            }

            log::debug!("{err}");
            return DEFAULT_BALANCE.to_string();
        }
    };

    let json_string = match to_string(&stored_value) {
        Ok(s) => s,
        Err(_) => {
            log::error!("Failed to serialize stored value into JSON.");
            return DEFAULT_BALANCE.to_string();
        }
    };
    stored_value_to_parsed_string(&json_string).unwrap_or_default()
}

pub async fn print_balance(
    maybe_vesting_type: Option<VestingType>,
    maybe_key: Option<Key>,
    maybe_contract: Option<Key>,
) {
    let mut key_info_map: IndexMap<String, IndexMap<String, String>> = IndexMap::new();
    let mut key_map = IndexMap::new();

    let balance_token = get_balance(
        maybe_vesting_type,
        maybe_key.clone().or_else(|| maybe_contract.clone()),
    )
    .await;

    let (balance, balance_motes) = get_cspr_balance_from_vesting_or_key(
        maybe_vesting_type,
        maybe_key.clone(),
        maybe_contract.clone(),
    )
    .await;

    key_map.insert("balance_motes".to_string(), balance_motes);

    key_map.insert(
        "balance_CSPR".to_string(),
        format_with_thousands_separator(&balance),
    );

    key_map.insert(
        format!("balance_{}", *COWL_CEP_18_COOL_SYMBOL),
        balance_token.clone(),
    );
    key_map.insert(
        format!("balance_{}", *COWL_CEP_18_TOKEN_SYMBOL),
        format_with_thousands_separator(&motes_to_cspr(&balance_token).unwrap()),
    );

    let identifier = maybe_vesting_type
        .map(|vesting_type| vesting_type.to_string())
        .or_else(|| maybe_key.clone().map(|key| key.to_formatted_string()))
        .or_else(|| maybe_contract.map(|contract| contract.to_formatted_string()))
        .unwrap_or_default();

    key_info_map.insert(identifier.clone(), key_map);

    let json_output = serde_json::to_string_pretty(&key_info_map).unwrap();
    log::info!("\n{}", json_output);
}

pub async fn get_cspr_account_balance(
    key_pair: Option<&KeyPair>,
    string_identifier: Option<String>,
    maybe_key: Option<Key>,
) -> (String, String) {
    let purse_identifier = if let Some(key) = key_pair {
        PurseIdentifier::from_main_purse_under_account_hash(
            key.public_key.clone().to_account_hash(),
        )
    } else if let Some(maybe_key) = maybe_key.clone() {
        PurseIdentifier::from_main_purse_under_account_hash(maybe_key.into_account().unwrap())
    } else {
        log::error!("No key_pair nor key are valid");
        return (DEFAULT_BALANCE.to_string(), DEFAULT_BALANCE.to_string());
    };
    let maybe_balance_motes = sdk()
        .query_balance(None, None, Some(purse_identifier), None, None, None, None)
        .await;

    let balance_motes = if let Ok(balance_motes) = maybe_balance_motes {
        balance_motes.result.balance.to_string()
    } else {
        let string_identifier_string = match string_identifier {
            Some(string_identifier) => string_identifier,
            None => maybe_key
                .clone()
                .map(|key| key.to_formatted_string())
                .unwrap_or_else(|| "Failed to retrieve account hash from key".to_string()),
        };

        if !string_identifier_string.is_empty() && key_pair.is_some() {
            log::warn!(
                "No CSPR balance for {}\n\
            - Private Key: {:?}\n\
            - Public Key: {}\n\
            - Account Hash: {}",
                string_identifier_string,
                key_pair.unwrap().private_key_base64,
                key_pair.unwrap().public_key.to_string(),
                key_pair
                    .unwrap()
                    .public_key
                    .to_account_hash()
                    .to_formatted_string()
            );
        } else {
            log::warn!(
                "No CSPR balance for\n\
            - Key: {}",
                string_identifier_string,
            );
        }
        DEFAULT_BALANCE.to_string()
    };
    let balance = motes_to_cspr(&balance_motes).unwrap();
    (balance, balance_motes)
}

pub async fn get_cspr_contract_balance(contract_package: &Key) -> (String, String) {
    let purse_uref = get_contract_swap_purse(contract_package)
        .await
        .expect("contract should have a purse Uref");

    let purse_identifier = PurseIdentifier::from_purse_uref(purse_uref);

    let maybe_balance_motes = sdk()
        .query_balance(None, None, Some(purse_identifier), None, None, None, None)
        .await;

    let balance_motes = if let Ok(balance_motes) = maybe_balance_motes {
        balance_motes.result.balance.to_string()
    } else {
        log::warn!(
            "No CSPR balance for Contract Package\n\
            - Key: {:?}\n",
            contract_package,
        );
        DEFAULT_BALANCE.to_string()
    };
    let balance = motes_to_cspr(&balance_motes).unwrap();
    (balance, balance_motes)
}

async fn get_cspr_balance_from_vesting_or_key(
    maybe_vesting_type: Option<VestingType>,
    maybe_key: Option<Key>,
    maybe_contract: Option<Key>,
) -> (String, String) {
    let default_balance = (DEFAULT_BALANCE.to_string(), DEFAULT_BALANCE.to_string());
    if maybe_contract.is_some() {
        return get_cspr_contract_balance(&maybe_contract.unwrap()).await;
    }
    match (maybe_vesting_type, maybe_key) {
        (Some(vesting_type), _) => {
            let key_pair = get_key_pair_from_vesting(&vesting_type.to_string())
                .await
                .unwrap();
            get_cspr_account_balance(Some(&key_pair), Some(vesting_type.to_string()), None).await
        }
        (_, Some(key)) => {
            let (vesting_type, key_pair) = get_key_pair_from_key(&key).await;
            match (vesting_type, key_pair) {
                (Some(vesting_type), Some(key_pair)) => {
                    get_cspr_account_balance(Some(&key_pair), Some(vesting_type), None).await
                }
                (None, Some(key_pair)) => {
                    get_cspr_account_balance(Some(&key_pair), None, None).await
                }
                _ => get_cspr_account_balance(None, None, Some(key)).await,
            }
        }
        _ => default_balance,
    }
}
