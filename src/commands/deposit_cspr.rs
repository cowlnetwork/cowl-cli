use crate::{
    commands::balance::print_balance,
    utils::{
        config::get_key_pair_from_vesting,
        constants::{
            CHAIN_NAME, COWL_DEPOSIT_CSPR_CALL_PAYMENT_AMOUNT, DEFAULT_SWAP_DEPOSIT_CSPR_SESSION,
            EVENTS_ADDRESS, INSTALLER, TTL, WASM_PATH,
        },
        format_with_thousands_separator, get_contract_swap_hash_keys,
        keys::format_base64_to_pem,
        prompt_yes_no, read_wasm_file, sdk,
    },
};
use casper_rust_wasm_sdk::{
    deploy_watcher::watcher::EventParseResult,
    helpers::motes_to_cspr,
    types::{
        deploy_hash::DeployHash,
        deploy_params::{deploy_str_params::DeployStrParams, session_str_params::SessionStrParams},
        key::Key,
    },
};
use cowl_swap::constants::ARG_COWL_SWAP_CONTRACT_PACKAGE;
use cowl_vesting::constants::ARG_AMOUNT;
use serde_json::json;
use std::process;

pub async fn deposit_cspr(amount: String) {
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
        "Please confirm deposit of {} {} ({} {})?",
        format_with_thousands_separator(&motes_to_cspr(&amount).unwrap()),
        "CSPR",
        amount,
        "motes",
    ));

    if !answer {
        log::warn!("Transfer aborted.");
        return;
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
    let path = &format!("{}{}.wasm", WASM_PATH, DEFAULT_SWAP_DEPOSIT_CSPR_SESSION);
    let module_bytes = match read_wasm_file(path) {
        Ok(module_bytes) => module_bytes,
        Err(err) => {
            log::error!("Error reading file {}: {:?}", path, err);
            return;
        }
    };
    session_params.set_session_bytes(module_bytes.into());

    let args_deposit_cspr_json = json!([
        {
            "name": ARG_COWL_SWAP_CONTRACT_PACKAGE,
            "type": "Key",
            "value": cowl_swap_contract_package_hash
        },
        {
            "name": ARG_AMOUNT,
            "type": "U512",
            "value": amount
        }
    ]);

    session_params.set_session_args_json(&args_deposit_cspr_json.to_string());

    let session_call = sdk()
        .install(
            deploy_params,
            session_params,
            &COWL_DEPOSIT_CSPR_CALL_PAYMENT_AMOUNT,
            None,
        )
        .await;

    let api_version = session_call
        .as_ref()
        .unwrap()
        .result
        .api_version
        .to_string();

    if api_version.is_empty() {
        log::error!("Failed to retrieve contract API version");
        process::exit(1)
    }

    let deploy_hash = DeployHash::from(
        session_call
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

    log::info!(
        "Wait deploy_hash for deposit {} {}",
        "CSPR",
        deploy_hash_as_string,
    );

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

pub async fn print_deposit_cspr(amount: String) {
    deposit_cspr(amount).await
}
