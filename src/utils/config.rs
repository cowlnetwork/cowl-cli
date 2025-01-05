use super::constants::{INSTALLER, USER_1, USER_2};
use crate::utils::keys::{fetch_funded_keys, insert_config_info, KeyPair};
use cowl_vesting::{enums::VESTING_INFO, vesting::VestingInfo};
use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

type KeyVestingInfoPair = (KeyPair, Option<VestingInfo>);
// Type alias for the HashMap structure
pub type ConfigInfo = HashMap<String, KeyVestingInfoPair>;
// Type alias for the Mutex-wrapped Option of the map
type ConfigInfoLock = Arc<Mutex<Option<ConfigInfo>>>;
// Lazy static variable to hold keys in memory
pub static CONFIG_LOCK: Lazy<ConfigInfoLock> = Lazy::new(|| Arc::new(Mutex::new(None)));

pub async fn init() {
    dotenv().ok();

    let mut funded_keys: VecDeque<KeyPair> = match fetch_funded_keys().await {
        Ok(keys) => keys,
        Err(e) => {
            log::error!("Error fetching keys: {}", e);
            return;
        }
    }
    .into();

    let mut config_info: ConfigInfo = HashMap::new();

    // Insert first 3 accounts (Installer, Account_1, Account_2) with no vesting info
    // Check environment variables and add keys accordingly
    let default_accounts = [INSTALLER, USER_1, USER_2];
    for &account_name in &default_accounts {
        insert_config_info(account_name, &mut funded_keys, &mut config_info, None);
    }

    // Insert vesting accounts
    for vesting_info in VESTING_INFO.iter() {
        insert_config_info(
            &vesting_info.vesting_type.to_string(),
            &mut funded_keys,
            &mut config_info,
            Some(vesting_info.clone()),
        );
    }

    // Store the key info in a mutex
    let mut config_lock = CONFIG_LOCK.lock().await;

    // Update the value inside the Option
    *config_lock = Some(config_info);

    // dbg!(config_lock); // Just to show the result
}

pub async fn get_key_pair_from_vesting(identifier: &str) -> Option<KeyPair> {
    // Acquire the lock and clone info
    let cloned_config = {
        let config_lock = CONFIG_LOCK.lock().await;
        config_lock.clone()
    };
    // Access the underlying ConfigInfo if available
    if let Some(ref config_info) = cloned_config {
        // Look up the KeyVestingInfoPair by vesting_type
        if let Some((key_pair, _)) = config_info.get(identifier) {
            return Some(key_pair.clone());
        }
    }
    // Return None if no matching vesting_type is found or config is None
    None
}
