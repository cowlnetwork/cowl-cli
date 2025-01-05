use super::balance::{get_balance, get_cspr_balance};
use crate::utils::{
    config::CONFIG_LOCK,
    constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL},
    format_with_thousands_separator,
};
use casper_rust_wasm_sdk::{helpers::motes_to_cspr, types::key::Key};
use indexmap::IndexMap;

pub async fn list_funded_addresses() -> Option<IndexMap<String, IndexMap<String, String>>> {
    // Acquire the lock and clone info
    let cloned_config = {
        let config_lock = CONFIG_LOCK.lock().await;
        config_lock.clone()
    };

    if let Some(config) = cloned_config.as_ref() {
        let mut key_info_map: IndexMap<String, IndexMap<String, String>> = IndexMap::new();
        for (vesting_type, (key_pair, _)) in config {
            let mut key_map = IndexMap::new();

            key_map.insert(
                "public_key".to_string(),
                key_pair.public_key.to_string().clone(),
            );
            key_map.insert(
                "account_hash".to_string(),
                key_pair
                    .public_key
                    .clone()
                    .to_account_hash()
                    .to_formatted_string(),
            );

            let (balance, balance_motes) = get_cspr_balance(key_pair, vesting_type).await;

            key_map.insert("balance_motes".to_string(), balance_motes);

            key_map.insert(
                "balance_CSPR".to_string(),
                format_with_thousands_separator(&balance),
            );

            let balance_token = get_balance(
                None,
                Some(Key::from_account(
                    key_pair.public_key.clone().to_account_hash(),
                )),
            )
            .await;

            key_map.insert(
                format!("balance_{}", *COWL_CEP_18_COOL_SYMBOL),
                balance_token.clone(),
            );
            key_map.insert(
                format!("balance_{}", *COWL_CEP_18_TOKEN_SYMBOL),
                format_with_thousands_separator(&motes_to_cspr(&balance_token).unwrap()),
            );

            key_info_map.insert(vesting_type.clone(), key_map);
        }

        // Sort key_info_map by its keys
        let sorted_key_info_map = sort_indexmap(key_info_map);

        Some(sorted_key_info_map)
    } else {
        None
    }
}

fn sort_indexmap<K: Ord + Clone + std::hash::Hash, V: Clone>(
    map: IndexMap<K, V>,
) -> IndexMap<K, V> {
    let mut sorted_entries: Vec<_> = map.into_iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(&b.0));
    sorted_entries.into_iter().collect()
}

pub async fn print_funded_addresses() {
    if let Some(key_info_map) = list_funded_addresses().await {
        let json_output = serde_json::to_string_pretty(&key_info_map).unwrap();
        log::info!("{}", json_output);
    }
}
