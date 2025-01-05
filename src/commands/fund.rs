use std::{process, str::FromStr};

use crate::{
    commands::balance::print_balance,
    utils::{
        config::get_key_pair_from_vesting,
        constants::{
            CHAIN_NAME, EVENTS_ADDRESS, INSTALLER, MINIMUM_TRANSFER_AMOUNT,
            PAYMENT_TRANSFER_AMOUNT, TTL,
        },
        format_with_thousands_separator,
        keys::format_base64_to_pem,
        prompt_yes_no, sdk,
    },
};
use bigdecimal::BigDecimal;
use casper_rust_wasm_sdk::{
    deploy_watcher::watcher::EventParseResult,
    helpers::motes_to_cspr,
    types::{
        deploy_hash::DeployHash,
        deploy_params::{deploy_str_params::DeployStrParams, payment_str_params::PaymentStrParams},
        key::Key,
    },
};
use cowl_vesting::enums::VestingType;

pub async fn fund_addresses(
    maybe_vesting_type: Option<VestingType>,
    maybe_key: Option<Key>,
    amount: String,
) {
    let target_account_hash = if let Some(vesting_type) = maybe_vesting_type {
        let key_pair = get_key_pair_from_vesting(&vesting_type.to_string())
            .await
            .unwrap();
        key_pair.public_key.to_account_hash().to_formatted_string()
    } else if let Some(key) = maybe_key.clone() {
        key.clone().into_account().unwrap().to_formatted_string()
    } else {
        log::error!("Both vesting_type and vesting_key are missing.");
        return;
    };

    // Retrieve the private key of Installer
    let key_pair = get_key_pair_from_vesting(INSTALLER).await.unwrap();

    let to = match maybe_vesting_type {
        Some(vesting_type) => vesting_type.to_string(),
        None => maybe_key
            .clone()
            .map(|key| key.to_formatted_string())
            .unwrap_or_else(|| "Failed to retrieve account hash".to_string()),
    };

    let answer = prompt_yes_no(&format!(
        "Please confirm funding of {} CSPR ({} motes) to {}?",
        format_with_thousands_separator(&motes_to_cspr(&amount).unwrap()),
        amount,
        &to
    ));
    if !answer {
        log::info!("You chose not to fund");
        process::exit(0);
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
    let payment_params = PaymentStrParams::default();
    payment_params.set_payment_amount(PAYMENT_TRANSFER_AMOUNT);

    let transfer = sdk()
        .transfer(
            &amount,
            &target_account_hash,
            None,
            deploy_params,
            payment_params,
            None,
            None,
        )
        .await;

    let api_version = transfer.as_ref().unwrap().result.api_version.to_string();

    if api_version.is_empty() {
        log::error!("Failed to retrieve contract API version");
        process::exit(1)
    }

    let deploy_hash = DeployHash::from(
        transfer
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

    log::info!("Wait deploy_hash for funding {}", deploy_hash_as_string);

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
}

pub async fn print_fund_addresses(
    vesting_type: Option<VestingType>,
    key: Option<Key>,
    amount: String,
) {
    if !check_amount(&amount) {
        log::error!(
            "Amount {} CSPR ({} motes) is less than minimum {} CSPR ({} motes)",
            format_with_thousands_separator(&motes_to_cspr(&amount).unwrap()),
            amount,
            format_with_thousands_separator(&motes_to_cspr(MINIMUM_TRANSFER_AMOUNT).unwrap()),
            MINIMUM_TRANSFER_AMOUNT
        );
        process::exit(1)
    }

    print_balance(vesting_type, key.clone()).await;

    fund_addresses(vesting_type, key.clone(), amount).await;

    print_balance(vesting_type, key.clone()).await;
}

fn check_amount(amount_str: &str) -> bool {
    // Parse the amount and threshold into BigDecimal
    let amount = BigDecimal::from_str(amount_str).expect("Invalid amount");
    let threshold = BigDecimal::from_str(MINIMUM_TRANSFER_AMOUNT).expect("Invalid threshold");

    // Compare the amounts
    amount >= threshold
}
