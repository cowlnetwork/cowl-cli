use crate::utils::{
    call_vesting_entry_point, get_contract_vesting_hash_keys, get_dictionary_item_params, sdk,
    stored_value_to_vesting_data,
};
use cowl_vesting::{
    constants::{DICT_VESTING_STATUS, ENTRY_POINT_VESTING_STATUS},
    enums::VestingType,
    vesting::VestingStatus,
};
use serde_json::to_string;

pub async fn vesting_status(
    vesting_type: VestingType,
    call_entry_point: bool,
) -> Option<VestingStatus> {
    // Retrieve contract vesting hash and package hash
    let (contract_vesting_hash, _) = match get_contract_vesting_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => {
            log::error!("Failed to retrieve contract vesting hash and package hash.");
            return None;
        }
    };

    if call_entry_point {
        call_vesting_entry_point(
            &contract_vesting_hash,
            ENTRY_POINT_VESTING_STATUS,
            vesting_type,
        )
        .await;
    }

    // Convert the vesting type to string for use in the dictionary lookup
    let dictionary_key = vesting_type.to_string();

    // Get the dictionary item parameters for the vesting status
    let dictionary_item = get_dictionary_item_params(
        &contract_vesting_hash.to_string(),
        DICT_VESTING_STATUS,
        &dictionary_key,
    );

    // Query the contract dictionary for the vesting status
    let vesting_status_result = sdk()
        .query_contract_dict(dictionary_item, None::<&str>, None, None)
        .await;

    // Handle query result and extract stored value
    let stored_value = match vesting_status_result {
        Ok(result) => result.result.stored_value,
        Err(_) => {
            log::error!("Failed to query vesting status from the contract.");
            return None;
        }
    };

    let json_string = match to_string(&stored_value) {
        Ok(s) => s,
        Err(_) => {
            log::error!("Failed to serialize stored value into JSON.");
            return None;
        }
    };

    stored_value_to_vesting_data(&json_string)
}

pub async fn print_vesting_status(vesting_type: VestingType, call_entry_point: bool) {
    if let Some(vesting_status) = vesting_status(vesting_type, call_entry_point).await {
        let json_output = serde_json::to_string_pretty(&vesting_status.to_string()).unwrap();
        log::info!("{}", json_output);
    }
}
