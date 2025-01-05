use crate::utils::{
    config::{get_key_pair_from_vesting, CONFIG_LOCK},
    constants::{
        CHAIN_NAME, COWL_CEP_18_INSTALL_PAYMENT_AMOUNT, COWL_CEP_18_TOKEN_DECIMALS,
        COWL_CEP_18_TOKEN_NAME, COWL_CEP_18_TOKEN_SYMBOL, COWL_VESTING_NAME,
        DEFAULT_COWL_CEP_18_TOKEN_DECIMALS, DEFAULT_COWL_CEP_18_TOKEN_NAME,
        DEFAULT_COWL_VESTING_NAME, EVENTS_ADDRESS, INSTALLER, TTL, WASM_PATH,
    },
    format_with_thousands_separator, get_contract_cep18_hash_keys, get_contract_vesting_hash_keys,
    keys::format_base64_to_pem,
    prompt_yes_no, read_wasm_file, sdk,
};
use casper_rust_wasm_sdk::{
    deploy_watcher::watcher::EventParseResult,
    helpers::motes_to_cspr,
    types::{
        deploy_hash::DeployHash,
        deploy_params::{deploy_str_params::DeployStrParams, session_str_params::SessionStrParams},
    },
};
use cowl_vesting::{
    constants::{ARG_COWL_CEP18_CONTRACT_PACKAGE, ARG_UPGRADE_FLAG},
    enums::EventsMode,
};
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::{io::Error, process};
use tokio::sync::Mutex;

const ARG_NAME: &str = "name";
const ARG_SYMBOL: &str = "symbol";
const ARG_DECIMALS: &str = "decimals";
const ARG_TOTAL_SUPPLY: &str = "total_supply";
const ARG_EVENTS_MODE: &str = "events_mode";
const ARG_ENABLE_MINT_BURN: &str = "enable_mint_burn";

static ARGS_CEP18_JSON: Lazy<Mutex<Value>> = Lazy::new(|| {
    Mutex::new(json!([
        {
            "name": ARG_NAME,
            "type": "String",
            "value": *COWL_CEP_18_TOKEN_NAME
        },
        {
            "name": ARG_SYMBOL,
            "type": "String",
            "value": *COWL_CEP_18_TOKEN_SYMBOL
        },
        {
            "name": ARG_DECIMALS,
            "type": "U8",
            "value": COWL_CEP_18_TOKEN_DECIMALS.parse::<u8>().unwrap_or(DEFAULT_COWL_CEP_18_TOKEN_DECIMALS)
        },
        {
            "name": ARG_TOTAL_SUPPLY,
            "type": "U8",
            "value": 0
        },
        {
            "name": ARG_EVENTS_MODE,
            "type": "U8",
            "value": EventsMode::CES as u8
        },
        {
            "name": ARG_ENABLE_MINT_BURN,
            "type": "Bool",
            "value": true
        }
    ]))
});

static ARGS_VESTING_JSON: Lazy<Mutex<Value>> = Lazy::new(|| {
    Mutex::new(json!([
        {
            "name": ARG_NAME,
            "type": "String",
            "value": *COWL_VESTING_NAME
        },
        {
            "name": ARG_EVENTS_MODE,
            "type": "U8",
            "value": EventsMode::CES as u8
        },
    ]))
});

pub async fn deploy_all_contracts() -> Result<(), Error> {
    deploy_cep18_token().await?;
    deploy_vesting_contract().await?;
    Ok(())
}

pub async fn deploy_cep18_token() -> Result<(), Error> {
    let key_pair = get_key_pair_from_vesting(INSTALLER).await.unwrap();

    let (contract_cep18_hash, _) = match get_contract_cep18_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => (String::from(""), String::from("")),
    };

    if !contract_cep18_hash.is_empty() {
        let answer = prompt_yes_no(&format!(
            "Token contract already exists at {}, do you want to upgrade?",
            contract_cep18_hash
        ));

        if answer {
            log::info!(
                "You chose to upgrade token contract {}",
                contract_cep18_hash
            );
            let mut args = ARGS_CEP18_JSON.lock().await;
            if let Some(array) = args.as_array_mut() {
                array.push(json!({
                    "name": ARG_UPGRADE_FLAG.to_string(),
                    "type": "Bool",
                    "value": true
                }));
            }
        } else {
            log::info!("You chose not to upgrade {}", contract_cep18_hash);
            return Ok(());
        }
    }

    let deploy_params = DeployStrParams::new(
        &CHAIN_NAME,
        &key_pair.public_key.to_string(),
        Some(format_base64_to_pem(
            &key_pair.private_key_base64.unwrap().clone(),
        )),
        None,
        Some(TTL.to_string()),
    );

    let session_params = SessionStrParams::default();
    let path = &format!("{}{}.wasm", WASM_PATH, DEFAULT_COWL_CEP_18_TOKEN_NAME);
    let module_bytes = match read_wasm_file(path) {
        Ok(module_bytes) => module_bytes,
        Err(err) => {
            log::error!("Error reading file {}: {:?}", path, err);
            return Err(err);
        }
    };
    session_params.set_session_bytes(module_bytes.into());
    session_params.set_session_args_json(&ARGS_CEP18_JSON.lock().await.to_string());

    let install = sdk()
        .install(
            deploy_params,
            session_params,
            &COWL_CEP_18_INSTALL_PAYMENT_AMOUNT,
            None,
        )
        .await;
    let api_version = install.as_ref().unwrap().result.api_version.to_string();

    if api_version.is_empty() {
        log::error!("Failed to retrieve contract API version");
        process::exit(1)
    }

    let deploy_hash = DeployHash::from(
        install
            .as_ref()
            .expect("should have a deploy hash")
            .result
            .deploy_hash,
    );
    let deploy_hash_as_string = deploy_hash.to_string();

    if deploy_hash_as_string.is_empty() {
        log::error!("Failed to retrieve deploy hash");
        process::exit(1)
    }

    if !contract_cep18_hash.is_empty() {
        log::info!(
            "Wait deploy_hash for token upgrade {}",
            deploy_hash_as_string
        );
    } else {
        log::info!(
            "Wait deploy_hash for token install {}",
            deploy_hash_as_string
        );
    }

    let event_parse_result: EventParseResult = sdk()
        .wait_deploy(&EVENTS_ADDRESS, &deploy_hash_as_string, None)
        .await
        .unwrap();

    let motes = event_parse_result
        .clone()
        .body
        .unwrap()
        .deploy_processed
        .unwrap()
        .execution_result
        .success
        .unwrap_or_else(|| {
            log::error!("Could not retrieved cost for deploy hash {deploy_hash_as_string}");
            log::error!("{:?}", &event_parse_result);
            process::exit(1)
        })
        .cost;

    let cost = format_with_thousands_separator(&motes_to_cspr(&motes).unwrap());

    let finalized_approvals = true;
    let get_deploy = sdk()
        .get_deploy(deploy_hash, Some(finalized_approvals), None, None)
        .await;
    let get_deploy = get_deploy.unwrap();
    let result = DeployHash::from(get_deploy.result.deploy.hash).to_string();
    log::info!("Processed deploy hash {result}");
    log::info!("Cost {cost} CSPR ({motes} motes)");
    let (contract_cep18_hash, contract_cep18_package_hash) =
        match get_contract_cep18_hash_keys().await {
            Some((hash, package_hash)) => (hash, package_hash),
            None => {
                log::error!("Failed to retrieve contract CEP18 keys");
                process::exit(1)
            }
        };
    log::info!("contract_cep18_hash {contract_cep18_hash}");
    log::info!("contract_cep18_package_hash {contract_cep18_package_hash}");
    Ok(())
}

pub async fn deploy_vesting_contract() -> Result<(), Error> {
    let key_pair = get_key_pair_from_vesting(INSTALLER).await.unwrap();

    let (cowl_cep18_token_contract_hash, cowl_cep18_token_package_hash) =
        match get_contract_cep18_hash_keys().await {
            Some((hash, package_hash)) => (hash, package_hash),
            None => (String::from(""), String::from("")),
        };

    if cowl_cep18_token_contract_hash.is_empty() {
        log::error!("Token contract does not exist in installer named keys at {cowl_cep18_token_contract_hash}");
        process::exit(1)
    }

    let (contract_vesting_hash, _) = match get_contract_vesting_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => (String::from(""), String::from("")),
    };

    if !contract_vesting_hash.is_empty() {
        let answer = prompt_yes_no(&format!(
            "Vesting contract already exists at {}, do you want to upgrade?",
            contract_vesting_hash
        ));

        if answer {
            log::info!(
                "You chose to upgrade vesting contract at {}",
                contract_vesting_hash
            );
            let mut args_vesting_json = ARGS_VESTING_JSON.lock().await;
            if let Some(array) = args_vesting_json.as_array_mut() {
                array.push(json!({
                    "name": ARG_UPGRADE_FLAG.to_string(),
                    "type": "Bool",
                    "value": true
                }));
            }
        } else {
            log::info!(
                "You chose not to upgrade vesting contract at {}",
                contract_vesting_hash
            );
            return Ok(());
        }
    }

    let deploy_params = DeployStrParams::new(
        &CHAIN_NAME,
        &key_pair.public_key.to_string(),
        Some(format_base64_to_pem(
            &key_pair.private_key_base64.unwrap().clone(),
        )),
        None,
        Some(TTL.to_string()),
    );

    let session_params = SessionStrParams::default();
    let path = &format!("{}{}.wasm", WASM_PATH, DEFAULT_COWL_VESTING_NAME);
    let module_bytes = match read_wasm_file(path) {
        Ok(module_bytes) => module_bytes,
        Err(err) => {
            log::error!("Error reading file {}: {:?}", path, err);
            return Err(err);
        }
    };
    session_params.set_session_bytes(module_bytes.into());

    {
        let mut args_vesting_json = ARGS_VESTING_JSON.lock().await;
        if contract_vesting_hash.is_empty() {
            let config_lock = CONFIG_LOCK.lock().await;
            if let Some(config) = config_lock.as_ref() {
                for (vesting_type, (key_pair, maybe_vesting_info)) in config {
                    if let Some(_vesting_info) = maybe_vesting_info {
                        if let Some(array) = args_vesting_json.as_array_mut() {
                            array.push(json!({
                                "name": *vesting_type,
                                "type": "Key",
                                "value": key_pair.public_key.to_account_hash().to_formatted_string()
                            }));
                        }
                    }
                }
            }
            drop(config_lock);
            if let Some(array) = args_vesting_json.as_array_mut() {
                array.push(json!({
                    "name": ARG_COWL_CEP18_CONTRACT_PACKAGE,
                    "type": "Key",
                    "value": cowl_cep18_token_package_hash
                }));
            }
        }
        session_params.set_session_args_json(&args_vesting_json.to_string());
    }

    let install = sdk()
        .install(
            deploy_params,
            session_params,
            &COWL_CEP_18_INSTALL_PAYMENT_AMOUNT,
            None,
        )
        .await;

    let api_version = install.as_ref().unwrap().result.api_version.to_string();

    if api_version.is_empty() {
        log::error!("Failed to retrieve contract API version");
        process::exit(1)
    }

    let deploy_hash = DeployHash::from(
        install
            .as_ref()
            .expect("should have a deploy hash")
            .result
            .deploy_hash,
    );
    let deploy_hash_as_string = deploy_hash.to_string();

    if deploy_hash_as_string.is_empty() {
        log::error!("Failed to retrieve deploy hash");
        process::exit(1)
    }

    if !contract_vesting_hash.is_empty() {
        log::info!(
            "Wait deploy_hash for vesting upgrade {}",
            deploy_hash_as_string
        );
    } else {
        log::info!(
            "Wait deploy_hash for vesting install {}",
            deploy_hash_as_string
        );
    }
    let event_parse_result: EventParseResult = sdk()
        .wait_deploy(&EVENTS_ADDRESS, &deploy_hash_as_string, None)
        .await
        .unwrap();
    let motes = event_parse_result
        .clone()
        .body
        .unwrap()
        .deploy_processed
        .unwrap()
        .execution_result
        .success
        .unwrap_or_else(|| {
            log::error!("Could not retrieved cost for deploy hash {deploy_hash_as_string}");
            log::error!("{:?}", &event_parse_result);
            process::exit(1)
        })
        .cost;

    let cost = format_with_thousands_separator(&motes_to_cspr(&motes).unwrap());

    let finalized_approvals = true;
    let get_deploy = sdk()
        .get_deploy(deploy_hash, Some(finalized_approvals), None, None)
        .await;
    let get_deploy = get_deploy.unwrap();
    let result = DeployHash::from(get_deploy.result.deploy.hash).to_string();
    log::info!("Processed deploy hash {result}");
    log::info!("Cost {cost} CSPR ({motes} motes)");
    let (contract_vesting_hash, contract_vesting_package_hash) =
        match get_contract_vesting_hash_keys().await {
            Some((hash, package_hash)) => (hash, package_hash),
            None => {
                log::error!("Failed to retrieve contract vesting keys");
                process::exit(1)
            }
        };
    log::info!("contract_vesting_hash {contract_vesting_hash}");
    log::info!("contract_vesting_package_hash {contract_vesting_package_hash}");
    Ok(())
}
