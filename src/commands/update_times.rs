use std::process;

use casper_rust_wasm_sdk::rpcs::query_global_state::{
    KeyIdentifierInput, PathIdentifierInput, QueryGlobalStateParams,
};
use chrono::{DateTime, TimeZone, Utc};
use cowl_swap::constants::{ARG_END_TIME, ARG_START_TIME};
use serde_json::{to_string, Value};

use crate::utils::{
    call_update_times, config::get_key_pair_from_vesting, constants::INSTALLER,
    get_contract_swap_hash_keys, prompt_yes_no, sdk,
};

pub async fn update_times(start_time: String, duration: String) -> (u64, u64) {
    let (cowl_swap_contract_hash, cowl_swap_contract_package_hash) =
        match get_contract_swap_hash_keys().await {
            Some((hash, package_hash)) => (hash, package_hash),
            None => (String::from(""), String::from("")),
        };

    if cowl_swap_contract_package_hash.is_empty() {
        log::error!("Swap contract package does not exist in installer named keys");
        process::exit(1)
    }

    let key_pair = get_key_pair_from_vesting(INSTALLER).await.unwrap();

    // Parse start_time and duration from strings to u64
    let start_time_secs: u64 = start_time.parse().expect("Invalid start_time format");
    let duration_secs: u64 = duration.parse().expect("Invalid duration format");

    // Calculate end_time
    let end_time_secs = start_time_secs + duration_secs;

    let start_time_datetime = timestamp_to_datetime(start_time_secs);
    let end_time_datetime = timestamp_to_datetime(end_time_secs);

    let answer = prompt_yes_no(&format!(
        "Please confirm update_times from {} to {}?",
        start_time_datetime, end_time_datetime
    ));

    if !answer {
        log::warn!("Setting times aborted.");
        return (0_u64, 0_u64);
    }

    // Call the update_times entry-point
    call_update_times(
        &key_pair,
        &cowl_swap_contract_package_hash,
        start_time_secs,
        duration_secs,
    )
    .await;

    // Query the actual start_time and end_time from the contract
    let start_time_query_params = QueryGlobalStateParams {
        key: KeyIdentifierInput::String(cowl_swap_contract_hash.clone()),
        path: Some(PathIdentifierInput::String(ARG_START_TIME.to_string())),
        maybe_global_state_identifier: None,
        state_root_hash: None,
        maybe_block_id: None,
        node_address: None,
        verbosity: None,
    };
    let actual_start_time = query_contract_key_as_u64(start_time_query_params).await;

    let end_time_query_params = QueryGlobalStateParams {
        key: KeyIdentifierInput::String(cowl_swap_contract_hash),
        path: Some(PathIdentifierInput::String(ARG_END_TIME.to_string())),
        maybe_global_state_identifier: None,
        state_root_hash: None,
        maybe_block_id: None,
        node_address: None,
        verbosity: None,
    };
    let actual_end_time = query_contract_key_as_u64(end_time_query_params).await;

    (actual_start_time, actual_end_time)
}

pub async fn print_update_times(start_time: String, duration: String) {
    log::info!("Update Times");
    let (actual_start_time, actual_end_time) = update_times(start_time, duration).await;

    let start_time_datetime = timestamp_to_datetime(actual_start_time);
    let end_time_datetime = timestamp_to_datetime(actual_end_time);

    log::info!(
        "START: {}_u64, date: {}",
        actual_start_time,
        start_time_datetime
    );
    log::info!("END: {}_u64, date: {}", actual_end_time, end_time_datetime);
}

// Function to convert a u64 timestamp to a human-readable DateTime<Utc>
fn timestamp_to_datetime(timestamp: u64) -> DateTime<Utc> {
    Utc.timestamp_opt(timestamp as i64, 0)
        .single()
        .expect("Invalid timestamp for DateTime conversion")
}

// Function to query a contract key and parse the result as a u64
async fn query_contract_key_as_u64(query_params: QueryGlobalStateParams) -> u64 {
    let query_contract_key = sdk().query_contract_key(query_params).await.unwrap();
    let json_string = to_string(&query_contract_key.result.stored_value).unwrap();
    let parsed_json: Value = serde_json::from_str(&json_string).unwrap();
    let cl_value_as_value = &parsed_json["CLValue"]["bytes"];
    let hex_string = cl_value_as_value
        .as_str()
        .expect("Expected a string in bytes field");

    u64::from_le_bytes(
        hex::decode(hex_string)
            .expect("Invalid hexadecimal format")
            .try_into()
            .expect("Hex string is not 8 bytes long"),
    )
}
