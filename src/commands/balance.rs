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
    let dictionary_key = match determine_dictionary_key(maybe_vesting_type, maybe_key.clone()).await
    {
        Ok(key) => key,
        Err(err) => {
            log::error!("{err}");
            return DEFAULT_BALANCE.to_string();
        }
    };

    let cowl_cep18_token_contract_hash = match fetch_contract_hash().await {
        Some(hash) => hash,
        None => return DEFAULT_BALANCE.to_string(),
    };

    let dictionary_item = get_dictionary_item_params(
        &cowl_cep18_token_contract_hash,
        DICT_BALANCES,
        &dictionary_key,
    );

    let balance_result = sdk()
        .query_contract_dict(dictionary_item, None::<&str>, None, None)
        .await;

    // Process the result directly
    match balance_result {
        Ok(response) => {
            let stored_value = &response.result.stored_value; // Assuming `stored_value` is what you need
            let json_string = to_string(stored_value).ok().unwrap();
            stored_value_to_parsed_string(&json_string).unwrap_or_default()
        }
        Err(err) => {
            log_balance_error(err.to_string(), maybe_vesting_type, maybe_key).await;
            DEFAULT_BALANCE.to_string()
        }
    }
}

async fn determine_dictionary_key(
    maybe_vesting_type: Option<VestingType>,
    maybe_key: Option<Key>,
) -> Result<String, String> {
    if let Some(vesting_type) = maybe_vesting_type {
        let key_pair = get_key_pair_from_vesting(&vesting_type.to_string())
            .await
            .ok_or_else(|| "Failed to retrieve key pair from vesting type".to_string())?;

        get_base64_key_from_account_hash(
            &key_pair.public_key.to_account_hash().to_formatted_string(),
        )
        .map_err(|err| {
            format!(
                "Failed to retrieve account_hash for {}: {:?}",
                vesting_type, err
            )
        })
    } else if let Some(key) = maybe_key {
        if key.to_formatted_string().contains("account") {
            get_base64_key_from_account_hash(
                &key.clone()
                    .into_account()
                    .ok_or_else(|| "get_balance method expects an account".to_string())?
                    .to_formatted_string(),
            )
            .map_err(|err| {
                format!(
                    "Failed to retrieve account hash for {}: {:?}",
                    key.to_formatted_string(),
                    err
                )
            })
        } else {
            get_base64_key_from_key_hash(&key.to_formatted_string()).map_err(|err| {
                format!(
                    "Failed to retrieve contract hash for {}: {:?}",
                    key.to_formatted_string(),
                    err
                )
            })
        }
    } else {
        Err("Both vesting_type and vesting_key are missing.".to_string())
    }
}

async fn fetch_contract_hash() -> Option<String> {
    match get_contract_cep18_hash_keys().await {
        Some((hash, _)) => Some(hash.to_string()),
        None => {
            log::error!("Failed to retrieve contract token hash and package hash.");
            None
        }
    }
}

async fn log_balance_error(
    err: String,
    maybe_vesting_type: Option<VestingType>,
    maybe_key: Option<Key>,
) {
    if let Some(vesting_type) = maybe_vesting_type {
        log::warn!(
            "No {} balance for {}!",
            *COWL_CEP_18_TOKEN_SYMBOL,
            vesting_type
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
    let purse_identifier = determine_purse_identifier(key_pair, maybe_key.clone());
    if purse_identifier.is_none() {
        log::error!("Neither key_pair nor maybe_key are valid");
        return (DEFAULT_BALANCE.to_string(), DEFAULT_BALANCE.to_string());
    }
    let purse_identifier = purse_identifier.unwrap();

    let maybe_balance_motes = sdk()
        .query_balance(None, None, Some(purse_identifier), None, None, None, None)
        .await;

    let balance_motes = match maybe_balance_motes {
        Ok(balance) => balance.result.balance.to_string(),
        Err(_) => handle_balance_error(key_pair, string_identifier, maybe_key),
    };

    let balance = motes_to_cspr(&balance_motes).unwrap_or(DEFAULT_BALANCE.to_string());
    (balance, balance_motes)
}

fn determine_purse_identifier(
    key_pair: Option<&KeyPair>,
    maybe_key: Option<Key>,
) -> Option<PurseIdentifier> {
    if let Some(key) = key_pair {
        Some(PurseIdentifier::from_main_purse_under_account_hash(
            key.public_key.clone().to_account_hash(),
        ))
    } else if let Some(key) = maybe_key {
        key.into_account()
            .map(PurseIdentifier::from_main_purse_under_account_hash)
    } else {
        None
    }
}

fn handle_balance_error(
    key_pair: Option<&KeyPair>,
    string_identifier: Option<String>,
    maybe_key: Option<Key>,
) -> String {
    let string_identifier_string = string_identifier.unwrap_or_else(|| {
        maybe_key
            .clone()
            .map(|key| key.to_formatted_string())
            .unwrap_or_else(|| "Failed to retrieve account hash from key".to_string())
    });

    if let Some(key) = key_pair {
        log::warn!(
            "No CSPR balance for {}\n\
            - Private Key: {:?}\n\
            - Public Key: {}\n\
            - Account Hash: {}",
            string_identifier_string,
            key.private_key_base64,
            key.public_key.to_string(),
            key.public_key.to_account_hash().to_formatted_string()
        );
    } else {
        log::warn!("No CSPR balance for\n- Key: {}", string_identifier_string);
    }

    DEFAULT_BALANCE.to_string()
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
    if let Some(contract) = maybe_contract {
        return get_cspr_contract_balance(&contract).await;
    }

    if let Some(vesting_type) = maybe_vesting_type {
        return handle_vesting_type_balance(vesting_type).await;
    }

    if let Some(key) = maybe_key {
        return handle_key_balance(key).await;
    }

    (DEFAULT_BALANCE.to_string(), DEFAULT_BALANCE.to_string())
}

async fn handle_vesting_type_balance(vesting_type: VestingType) -> (String, String) {
    let key_pair = get_key_pair_from_vesting(&vesting_type.to_string())
        .await
        .unwrap();
    get_cspr_account_balance(Some(&key_pair), Some(vesting_type.to_string()), None).await
}

async fn handle_key_balance(key: Key) -> (String, String) {
    let (maybe_vesting_type, maybe_key_pair) = get_key_pair_from_key(&key).await;

    match (maybe_vesting_type, maybe_key_pair) {
        (Some(vesting_type), Some(key_pair)) => {
            get_cspr_account_balance(Some(&key_pair), Some(vesting_type), None).await
        }
        (None, Some(key_pair)) => get_cspr_account_balance(Some(&key_pair), None, None).await,
        _ => get_cspr_account_balance(None, None, Some(key)).await,
    }
}
