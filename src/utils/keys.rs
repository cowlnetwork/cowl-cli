use casper_rust_wasm_sdk::{
    helpers::public_key_from_secret_key,
    types::{key::Key, public_key::PublicKey},
};
use cowl_vesting::{enums::VestingType, vesting::VestingInfo};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, env, error::Error, fs, path::Path};

use super::{
    config::{ConfigInfo, CONFIG_LOCK},
    constants::{FUNDED_KEYS_JSON_FILE_PATH, FUNDED_KEYS_URL},
};

const BEGIN_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----";
const END_PRIVATE_KEY: &str = "-----END PRIVATE KEY-----";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyPair {
    #[serde(rename = "private")]
    pub private_key_base64: Option<String>,
    #[serde(rename = "public")]
    pub public_key: PublicKey,
}

pub async fn fetch_funded_keys() -> Result<Vec<KeyPair>, Box<dyn Error>> {
    // Step 1: Check if the JSON file exists
    if !Path::new(FUNDED_KEYS_JSON_FILE_PATH).exists() {
        // Fetch the TS file content directly without saving it to disk
        let ts_content = match fetch_ts_file(FUNDED_KEYS_URL).await {
            Ok(content) => content,
            Err(e) => {
                log::error!("Error fetching TS file: {}", e);
                return Err(e);
            }
        };

        // Step 2: Parse the FUNDED_KEYS from the fetched TS content
        let funded_keys = match parse_funded_keys_from_content(&ts_content) {
            Ok(keys) => keys,
            Err(e) => {
                log::error!("Error parsing FUNDED_KEYS: {}", e);
                return Err(e);
            }
        };

        // Step 3: Write the parsed keys to the JSON file
        if let Err(e) = write_keys_to_json(&funded_keys, FUNDED_KEYS_JSON_FILE_PATH) {
            log::error!("Error writing keys to json: {}", e);
            return Err(e);
        } else {
            log::info!(
                "Keys successfully written to {}",
                FUNDED_KEYS_JSON_FILE_PATH
            );
        }

        Ok(funded_keys)
    } else {
        // Step 4: If the JSON file exists, load keys from it
        match load_keys_from_json(FUNDED_KEYS_JSON_FILE_PATH) {
            Ok(keys) => Ok(keys),
            Err(e) => {
                log::error!("Error loading keys from JSON file: {}", e);
                Err(e)
            }
        }
    }
}

pub async fn fetch_ts_file(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let body = response.text().await?;
    Ok(body)
}

pub fn parse_funded_keys_from_content(
    ts_content: &str,
) -> Result<Vec<KeyPair>, Box<dyn std::error::Error>> {
    // Extract the FUNDED_KEYS section
    let funded_keys_start = ts_content
        .find("FUNDED_KEYS = [")
        .ok_or("FUNDED_KEYS not found")?;
    let funded_keys_end = ts_content[funded_keys_start..]
        .find("];")
        .ok_or("FUNDED_KEYS not properly terminated")?
        + funded_keys_start;

    let funded_keys_text = &ts_content[funded_keys_start + 14..funded_keys_end + 1]; // Exclude "FUNDED_KEYS = " and include closing bracket
    let mut cleaned_keys_text = funded_keys_text
        .replace("private:", "\"private\":")
        .replace("public:", "\"public\":");

    // Regex to remove trailing commas
    let trailing_comma_regex = Regex::new(r",\s*(\}|\])")?;
    cleaned_keys_text = trailing_comma_regex
        .replace_all(&cleaned_keys_text, "$1")
        .to_string();

    // Parse the cleaned keys text into a Vec<Key> using serde_json
    let funded_keys: Vec<KeyPair> = serde_json::from_str(&cleaned_keys_text)?;
    Ok(funded_keys)
}

#[cfg(feature = "std-fs-io")]
fn write_keys_to_json(keys: &[KeyPair], file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(keys).unwrap();
    fs::write(file_path, json_string)?;
    Ok(())
}

#[cfg(feature = "std-fs-io")]
fn load_keys_from_json(file_path: &str) -> Result<Vec<KeyPair>, Box<dyn Error>> {
    let json_string = fs::read_to_string(file_path)?;
    let keys: Vec<KeyPair> = serde_json::from_str(&json_string)?;
    Ok(keys)
}

pub fn insert_config_info(
    identifier: &str,
    funded_keys: &mut VecDeque<KeyPair>,
    config_info: &mut ConfigInfo,
    vesting_info: Option<VestingInfo>,
) {
    // Always pop a key from funded_keys
    if let Some(default_key) = funded_keys.pop_front() {
        // Try to load the key from environment variables or file
        if load_key_from_env_or_file(identifier, vesting_info.clone(), config_info) {
            return; // Key was successfully loaded from env or file
        }

        // Insert the default key if no environment variable is found except if VestingType
        let private_key_base64 = match VestingType::try_from(identifier) {
            Ok(_vesting_type) => {
                None // Return `None` because it's a Vesting Address
            }
            Err(_) => {
                default_key.private_key_base64 // Use the private key
            }
        };

        // Check for env/file-based public key if VestingType
        let public_key = if VestingType::try_from(identifier).is_ok() {
            // Attempt to fetch an inline public key from the environment
            env::var(format!("PUBLIC_KEY_{}", identifier.to_uppercase()))
                .ok()
                .and_then(|key| PublicKey::new(&key).ok())
                .map_or_else(|| default_key.public_key.to_string(), |key| key.to_string())
        } else {
            // Use the default key for non-VestingType identifiers
            default_key.public_key.to_string()
        };

        config_info.insert(
            identifier.to_string(),
            (
                KeyPair {
                    private_key_base64,
                    public_key: PublicKey::new(&public_key).unwrap(),
                },
                vesting_info,
            ),
        );
    }
}

fn load_key_from_env_or_file(
    identifier: &str,
    vesting_info: Option<VestingInfo>,
    config_info: &mut ConfigInfo,
) -> bool {
    // Helper to try loading a private key and inserting it into `config_info`
    let mut try_insert = |private_key: String| -> bool {
        if let Ok(public_key_hex) = public_key_from_secret_key(&private_key) {
            // Clean the private key by removing the header, footer, and surrounding whitespace
            let cleaned_private_key = private_key
                .replace(BEGIN_PRIVATE_KEY, "")
                .replace(END_PRIVATE_KEY, "")
                .trim()
                .to_string();

            config_info.insert(
                identifier.to_string(),
                (
                    KeyPair {
                        private_key_base64: Some(cleaned_private_key),
                        public_key: PublicKey::new(&public_key_hex).unwrap(),
                    },
                    vesting_info.clone(),
                ),
            );
            return true;
        }
        false
    };

    // Check for file-based private key in env
    if let Ok(key_file_path) = env::var(format!("PATH_PRIVATE_KEY_{}", identifier.to_uppercase())) {
        if let Ok(private_key) = std::fs::read_to_string(&key_file_path) {
            return try_insert(private_key);
        }
    }

    // Check for inline private key in env
    if let Ok(private_key) = env::var(format!("PRIVATE_KEY_{}", identifier.to_uppercase())) {
        return try_insert(private_key);
    }

    false
}

pub fn format_base64_to_pem(private_key: &str) -> String {
    format!("{BEGIN_PRIVATE_KEY} {private_key} {END_PRIVATE_KEY}")
}

fn prompt_base64_or_path(prompt_message: &str) -> String {
    println!("{}", prompt_message);
    println!("Enter the base64 key directly or specify the file path:");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

fn process_base64_or_path(input: String) -> String {
    if std::fs::metadata(&input).is_ok() {
        // Treat as file path and read content
        std::fs::read_to_string(&input).expect("Failed to read file")
    } else {
        // Treat as direct base64 input
        format_base64_to_pem(&input)
    }
}

pub async fn retrieve_private_key(public_key: &PublicKey) -> Option<String> {
    match get_private_key_base64(public_key).await {
        Some(key) => Some(key),
        None => {
            let answer = prompt_base64_or_path(&format!(
                "Missing private key for {}, do you want to provide a base64 string or a .pem file path?",
                public_key
            ));
            let processed_key = process_base64_or_path(answer);
            if processed_key.is_empty() {
                log::warn!("No valid private key provided.");
                None
            } else {
                Some(processed_key)
            }
        }
    }
}

async fn get_private_key_base64(public_key: &PublicKey) -> Option<String> {
    let config_lock = CONFIG_LOCK.lock().await.clone().unwrap();

    for (_, (key_pair, _)) in config_lock.iter() {
        if &key_pair.public_key == public_key {
            return key_pair
                .private_key_base64
                .as_ref()
                .map(|base64_key| format_base64_to_pem(&base64_key.clone()));
        }
    }
    None
}

pub async fn get_key_pair_from_key(account_hash: &Key) -> (Option<String>, Option<KeyPair>) {
    let config_lock = CONFIG_LOCK.lock().await.clone().unwrap();

    for (vesting_type, (key_pair, _)) in config_lock.iter() {
        if key_pair.public_key.to_account_hash().to_formatted_string()
            == account_hash.to_formatted_string()
        {
            return (Some(vesting_type.to_string()), Some(key_pair.clone()));
        }
    }
    (None, None)
}
