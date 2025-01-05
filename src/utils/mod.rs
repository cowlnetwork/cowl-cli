use bigdecimal::BigDecimal;
use casper_rust_wasm_sdk::deploy_watcher::watcher::EventParseResult;
use casper_rust_wasm_sdk::helpers::motes_to_cspr;
use casper_rust_wasm_sdk::rpcs::get_dictionary_item::DictionaryItemInput;
use casper_rust_wasm_sdk::rpcs::query_global_state::{KeyIdentifierInput, QueryGlobalStateParams};
use casper_rust_wasm_sdk::types::deploy_hash::DeployHash;
use casper_rust_wasm_sdk::types::deploy_params::deploy_str_params::DeployStrParams;
use casper_rust_wasm_sdk::types::deploy_params::dictionary_item_str_params::DictionaryItemStrParams;
use casper_rust_wasm_sdk::types::deploy_params::payment_str_params::PaymentStrParams;
use casper_rust_wasm_sdk::types::deploy_params::session_str_params::SessionStrParams;
use casper_rust_wasm_sdk::types::key::Key;
use casper_rust_wasm_sdk::types::public_key::PublicKey;
use casper_rust_wasm_sdk::{types::verbosity::Verbosity, SDK};
use config::get_key_pair_from_vesting;
use constants::{
    CHAIN_NAME, COWL_CEP18_TOKEN_CONTRACT_HASH_NAME, COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME,
    COWL_SET_MODALITIES_CALL_PAYMENT_AMOUNT, COWL_TOKEN_TRANSFER_CALL_PAYMENT_AMOUNT,
    COWL_VESTING_CALL_PAYMENT_AMOUNT, COWL_VESTING_NAME, EVENTS_ADDRESS, INSTALLER, RPC_ADDRESS,
    TTL,
};
use cowl_vesting::constants::{
    ARG_AMOUNT, ARG_EVENTS_MODE, ARG_OWNER, ARG_RECIPIENT, ARG_SPENDER, ARG_VESTING_TYPE,
    ENTRY_POINT_DECREASE_ALLOWANCE, ENTRY_POINT_INCREASE_ALLOWANCE, ENTRY_POINT_SET_MODALITIES,
    ENTRY_POINT_TRANSFER, ENTRY_POINT_TRANSFER_FROM, PREFIX_CONTRACT_NAME,
    PREFIX_CONTRACT_PACKAGE_NAME,
};
use cowl_vesting::enums::{EventsMode, VestingType};
use cowl_vesting::vesting::VestingData;
use keys::format_base64_to_pem;
use num_format::{Locale, ToFormattedString};
use once_cell::sync::Lazy;
use serde_json::{json, to_string, Value};
use std::io::Write;
use std::process;
use std::str::FromStr;
use std::{
    env,
    fs::File,
    io::{self, Read},
    sync::{Arc, Mutex},
};

pub mod config;
pub mod constants;
pub mod keys;

pub static SDK_INSTANCE: Lazy<Mutex<Option<Arc<SDK>>>> = Lazy::new(|| Mutex::new(None));

// Function to retrieve or create the SDK instance
pub fn sdk() -> Arc<SDK> {
    let mut instance = SDK_INSTANCE.lock().unwrap();
    if instance.is_none() {
        let new_sdk = SDK::new(Some(RPC_ADDRESS.to_string()), Some(Verbosity::High));
        *instance = Some(Arc::new(new_sdk));
    }
    instance.clone().unwrap()
}

pub fn read_wasm_file(file_path: &str) -> Result<Vec<u8>, io::Error> {
    let path_buf = env::current_dir()?;
    let mut relative_path_buf = path_buf.clone();
    relative_path_buf.push(file_path);
    let mut file = File::open(relative_path_buf)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn prompt_yes_no(question: &str) -> bool {
    loop {
        log::warn!("{} (y/n): ", question);
        io::stdout().flush().unwrap(); // Ensure the prompt is printed

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please answer with 'y' or 'n'"),
        }
    }
}

async fn get_contract_hash_keys(
    public_key: &PublicKey,
    contract_name: &str,
    contract_package_name: &str,
) -> Option<(String, String)> {
    let query_params: QueryGlobalStateParams = QueryGlobalStateParams {
        key: KeyIdentifierInput::String(public_key.to_account_hash().to_formatted_string()),
        path: None,
        maybe_global_state_identifier: None,
        state_root_hash: None,
        maybe_block_id: None,
        node_address: None,
        verbosity: None,
    };

    let query_global_state = sdk().query_global_state(query_params).await;
    let query_global_state_result = query_global_state.unwrap_or_else(|_| {
        panic!("Failed to query global state");
    });

    let json_string = to_string(&query_global_state_result.result.stored_value)
        .unwrap_or_else(|_| panic!("Failed to convert stored value to string"));

    let parsed_json: Value =
        serde_json::from_str(&json_string).unwrap_or_else(|_| panic!("Failed to parse JSON"));

    let named_keys = parsed_json["Account"]["named_keys"]
        .as_array()
        .unwrap_or_else(|| panic!("named_keys is not an array"));

    // Find the contract hash
    let contract_hash = named_keys
        .iter()
        .find(|obj| obj["name"] == Value::String(contract_name.to_string()))
        .and_then(|obj| obj["key"].as_str())
        .unwrap_or_else(|| {
            log::debug!("Contract hash key not found in named_keys");
            ""
        });

    if contract_hash.is_empty() {
        return None;
    }

    // Find the contract package hash
    let contract_package_hash = named_keys
        .iter()
        .find(|obj| obj["name"] == Value::String(contract_package_name.to_string()))
        .and_then(|obj| obj["key"].as_str())
        .unwrap_or_else(|| {
            log::error!("Package hash key not found in named_keys");
            ""
        });

    if contract_package_hash.is_empty() {
        return None;
    }

    Some((contract_hash.to_string(), contract_package_hash.to_string()))
}

// Specific function for getting CEP18 contract hash keys
pub async fn get_contract_cep18_hash_keys() -> Option<(String, String)> {
    let public_key = get_key_pair_from_vesting(INSTALLER)
        .await
        .unwrap()
        .public_key;
    get_contract_hash_keys(
        &public_key,
        &COWL_CEP18_TOKEN_CONTRACT_HASH_NAME,
        &COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME,
    )
    .await
}

// Specific function for getting Vesting contract hash keys
pub async fn get_contract_vesting_hash_keys() -> Option<(String, String)> {
    let public_key = get_key_pair_from_vesting(INSTALLER)
        .await
        .unwrap()
        .public_key;
    get_contract_hash_keys(
        &public_key,
        &format!("{PREFIX_CONTRACT_NAME}_{}", *COWL_VESTING_NAME),
        &format!("{PREFIX_CONTRACT_PACKAGE_NAME}_{}", *COWL_VESTING_NAME),
    )
    .await
}

pub fn get_dictionary_item_params(
    key: &str,
    dictionary_name: &str,
    dictionary_item_key: &str,
) -> DictionaryItemInput {
    let mut params = DictionaryItemStrParams::new();
    params.set_contract_named_key(key, dictionary_name, dictionary_item_key);
    DictionaryItemInput::Params(params)
}

pub fn stored_value_to_vesting_data<T>(json_string: &str) -> Option<T>
where
    T: VestingData,
{
    // Parse the JSON string
    let parsed_json: Value = match serde_json::from_str(json_string) {
        Ok(v) => v,
        Err(_) => {
            log::error!("Failed to parse JSON string.");
            return None;
        }
    };

    // Extract the "bytes" field from parsed JSON
    let cl_value_as_value = &parsed_json["CLValue"]["bytes"];

    // Check if the "bytes" field exists and is a valid string
    if let Some(hex_string) = cl_value_as_value.as_str() {
        // Decode the hex string to raw bytes
        let raw_bytes = match hex::decode(hex_string) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("Failed to decode hex string: {}", e);
                return None;
            }
        };

        // Attempt to deserialize the raw bytes into T
        match T::from_bytes(&raw_bytes) {
            Ok((info, _)) => Some(info),
            Err(e) => {
                log::error!("Error parsing bytes into VestingInfo: {:?}", e);
                None
            }
        }
    } else {
        log::error!("Expected 'bytes' field to be a string in JSON.");
        None
    }
}

pub fn stored_value_to_parsed_string(json_string: &str) -> Option<String> {
    // Parse the JSON string
    let parsed_json: Value = match serde_json::from_str(json_string) {
        Ok(v) => v,
        Err(_) => {
            log::error!("Failed to parse JSON string.");
            return None;
        }
    };

    let parsed = &parsed_json["CLValue"]["parsed"];

    // Try using the `parsed` field directly if it exists
    if let Some(parsed_value) = parsed.as_str() {
        return Some(parsed_value.to_string());
    }
    None
}

async fn execute_contract_entry_point(
    contract_hash: &str,
    entry_point: &str,
    args_json: &str,
    payment_amount: &str,
    public_key: &PublicKey,
    secret_key: String,
) -> (String, String) {
    let deploy_params = DeployStrParams::new(
        &CHAIN_NAME,
        &public_key.to_string(),
        Some(secret_key),
        None,
        Some(TTL.to_string()),
    );

    let session_params = SessionStrParams::default();
    session_params.set_session_hash(contract_hash);
    session_params.set_session_entry_point(entry_point);
    session_params.set_session_args_json(args_json);

    let payment_params = PaymentStrParams::default();
    payment_params.set_payment_amount(payment_amount);

    // Call the entry point
    let result = sdk()
        .call_entrypoint(deploy_params, session_params, payment_params, None)
        .await;

    let deploy_hash = DeployHash::from(
        result
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
        "Wait deploy_hash for entry point {} {}",
        entry_point,
        deploy_hash_as_string
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
            log::error!("Could not retrieve cost for deploy hash {deploy_hash_as_string}");
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

    (result, cost)
}

pub async fn call_vesting_entry_point(
    contract_vesting_hash: &str,
    entry_point: &str,
    vesting_type: VestingType,
) {
    let key_pair = get_key_pair_from_vesting(INSTALLER).await.unwrap();
    let args = json!([
        {
            "name": ARG_VESTING_TYPE,
            "type": "String",
            "value": vesting_type.to_string()
        },
    ])
    .to_string();

    execute_contract_entry_point(
        contract_vesting_hash,
        entry_point,
        &args,
        &COWL_VESTING_CALL_PAYMENT_AMOUNT,
        &key_pair.public_key,
        format_base64_to_pem(&key_pair.private_key_base64.unwrap()),
    )
    .await;
}

pub async fn call_set_modalities_entry_point(
    contract_vesting_hash: &str,
    events_mdoe: EventsMode,
) -> (String, String) {
    let key_pair = get_key_pair_from_vesting(INSTALLER).await.unwrap();
    let args = json!([
        {
            "name": ARG_EVENTS_MODE,
            "type": "U8",
            "value": events_mdoe as u8
        },
    ])
    .to_string();

    execute_contract_entry_point(
        contract_vesting_hash,
        ENTRY_POINT_SET_MODALITIES,
        &args,
        &COWL_SET_MODALITIES_CALL_PAYMENT_AMOUNT,
        &key_pair.public_key,
        format_base64_to_pem(&key_pair.private_key_base64.unwrap()),
    )
    .await
}

pub async fn call_token_transfer_entry_point(
    contract_token_hash: &str,
    public_key: &PublicKey,
    secret_key: String,
    from: Option<Key>,
    to: &Key,
    amount: String,
) {
    let mut args = json!([
        {
            "name": ARG_RECIPIENT,
            "type": "Key",
            "value": to.to_formatted_string()
        },
        {
            "name": ARG_AMOUNT,
            "type": "U256",
            "value": amount
        },
    ]);

    let entry_point = if let Some(from_key) = from {
        args.as_array_mut().unwrap().push(serde_json::json!({
            "name": ARG_OWNER,
            "type": "Key",
            "value": from_key.to_formatted_string()
        }));
        ENTRY_POINT_TRANSFER_FROM
    } else {
        ENTRY_POINT_TRANSFER
    };

    execute_contract_entry_point(
        contract_token_hash,
        entry_point,
        &args.to_string(),
        &COWL_TOKEN_TRANSFER_CALL_PAYMENT_AMOUNT,
        public_key,
        secret_key,
    )
    .await;
}

pub async fn call_token_set_allowance_entry_point(
    contract_token_hash: &str,
    public_key: &PublicKey,
    secret_key: String,
    spender: &Key,
    amount: String,
    decrease: bool,
) {
    let args = json!([
        {
            "name": ARG_SPENDER,
            "type": "Key",
            "value": spender.to_formatted_string()
        },
        {
            "name": ARG_AMOUNT,
            "type": "U256",
            "value": amount
        },
    ])
    .to_string();

    let entry_point = if decrease {
        ENTRY_POINT_DECREASE_ALLOWANCE
    } else {
        ENTRY_POINT_INCREASE_ALLOWANCE
    };

    execute_contract_entry_point(
        contract_token_hash,
        entry_point,
        &args,
        &COWL_TOKEN_TRANSFER_CALL_PAYMENT_AMOUNT,
        public_key,
        secret_key,
    )
    .await;
}

/// Formats a large number string with thousands separators and two decimal places.
///
/// # Arguments
/// * `number_str` - A number represented as a `&str`.
///
/// # Returns
/// * A `String` representing the formatted number, or "Invalid number" if parsing fails.
pub fn format_with_thousands_separator(number_str: &str) -> String {
    // Parse the input string into a BigDecimal
    let number = match BigDecimal::from_str(number_str) {
        Ok(n) => n,
        Err(_) => return "Invalid number".to_string(),
    };

    // Round to two decimal places
    let rounded = number.round(2);

    // Separate the integer and fractional parts
    let binding = rounded.to_string();
    let parts: Vec<&str> = binding.split('.').collect();
    let integer_part = parts.first().unwrap_or(&"0");
    let decimal_part = parts.get(1).unwrap_or(&"00");

    // Parse the integer part into a BigInt for formatting
    let integer_bigint = match integer_part.parse::<u128>() {
        Ok(n) => n,
        Err(_) => return "Invalid integer".to_string(),
    };
    let formatted_integer = integer_bigint.to_formatted_string(&Locale::en);

    // Combine the formatted integer and fractional parts
    format!("{}.{}", formatted_integer, decimal_part)
}
