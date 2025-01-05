use cowl_vesting::enums::EventsMode;

use super::deploy::deploy_vesting_contract;
use crate::utils::{call_set_modalities_entry_point, get_contract_vesting_hash_keys};

pub async fn upgrade_events() -> Option<String> {
    // Upgrade the vesting contract
    if deploy_vesting_contract().await.is_ok() {
        // Call the set modalities entry point
        let (contract_vesting_hash, _) = match get_contract_vesting_hash_keys().await {
            Some((hash, package_hash)) => (hash, package_hash),
            None => (String::from(""), String::from("")),
        };

        log::info!(
            "Vesting contract deemed to be upgraded at {}",
            contract_vesting_hash
        );
        log::info!("Calling set_modalities to enable events.");

        match call_set_modalities_entry_point(&contract_vesting_hash, EventsMode::CES).await {
            (success_message, _) if !success_message.is_empty() => {
                Some("Events mode enabled".to_string())
            }
            _ => {
                log::error!("Failed to enable events.");
                None
            }
        }
    } else {
        log::error!("Contract vesting not upgraded");
        None
    }
}

pub async fn print_upgrade_events() {
    if let Some(key_info) = upgrade_events().await {
        log::info!("{}", key_info);
    } else {
        log::error!("Failed to upgrade and enable events.");
    }
}
