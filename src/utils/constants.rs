use once_cell::sync::Lazy;
use std::env;

// All those following can be overifden by .env
const DEFAULT_RPC_ADDRESS: &str = "http://127.0.0.1:7777";
const DEFAULT_EVENTS_ADDRESS: &str = "http://127.0.0.1:9999/events/main";
const DEFAULT_CHAIN_NAME: &str = "casper-net-1";
const DEFAULT_TTL: &str = "30m";
const DEFAULT_CEP_18_TOKEN_SYMBOL: &str = "COWL";
const DEFAULT_CEP_18_COOL_SYMBOL: &str = "cool";
pub const DEFAULT_CEP_18_TOKEN_DECIMALS: u8 = 9;
pub const DEFAULT_CEP_18_TOKEN_NAME: &str = "cowl_cep18";
pub const DEFAULT_VESTING_NAME: &str = "cowl_vesting";
pub const DEFAULT_SWAP_NAME: &str = "cowl_swap";

const DEFAULT_CEP_18_INSTALL_PAYMENT_AMOUNT: &str = "300000000000"; // 300 CSPR
const DEFAULT_VESTING_INSTALL_PAYMENT_AMOUNT: &str = "300000000000"; // 300 CSPR
const DEFAULT_SWAP_INSTALL_PAYMENT_AMOUNT: &str = "300000000000"; // 300 CSPR
const DEFAULT_VESTING_CALL_PAYMENT_AMOUNT: &str = "350000000"; // 0.35 CSPR
const DEFAULT_SET_MODALITIES_CALL_PAYMENT_AMOUNT: &str = "1200000000"; // 1.2 CSPR
const DEFAULT_TOKEN_TRANSFER_CALL_PAYMENT_AMOUNT: &str = "2500000000"; // 2.5 CSPR
const DEFAULT_DEPOSIT_COWL_CALL_PAYMENT_AMOUNT: &str = "1800000000"; // 1.8 CSPR
const DEFAULT_DEPOSIT_CSPR_CALL_PAYMENT_AMOUNT: &str = "3100000000"; // 3.1 CSPR
const DEFAULT_WITHDRAW_COWL_CALL_PAYMENT_AMOUNT: &str = "1500000000"; // 1.5 CSPR
const DEFAULT_WITHDRAW_CSPR_CALL_PAYMENT_AMOUNT: &str = "3000000000"; // 3 CSPR
const DEFAULT_CSPR_TO_COWL_CALL_PAYMENT_AMOUNT: &str = "4400000000"; // 4.4 CSPR
const DEFAULT_COWL_TO_CSPR_CALL_PAYMENT_AMOUNT: &str = "5300000000"; // 5.3 CSPR
const DEFAULT_UPDATE_TIMES_CALL_PAYMENT_AMOUNT: &str = "500000000"; // 0.5 CSPR

pub const PAYMENT_TRANSFER_AMOUNT: &str = "100000000"; // 0.10 CSPR
pub const MINIMUM_TRANSFER_AMOUNT: &str = "2500000000"; // 2.5 CSPR
pub const DEFAULT_BALANCE: &str = "0";

pub const DEFAULT_SWAP_DEPOSIT_COWL_SESSION: &str = "deposit_cowl_session";
pub const DEFAULT_SWAP_DEPOSIT_CSPR_SESSION: &str = "deposit_cspr_session";
pub const DEFAULT_SWAP_COWL_TO_CSPR_SESSION: &str = "cowl_to_cspr_session";
pub const DEFAULT_SWAP_CSPR_TO_COWL_SESSION: &str = "cspr_to_cowl_session";

pub static RPC_ADDRESS: Lazy<String> =
    Lazy::new(|| env::var("RPC_ADDRESS").unwrap_or_else(|_| DEFAULT_RPC_ADDRESS.to_string()));
pub static EVENTS_ADDRESS: Lazy<String> =
    Lazy::new(|| env::var("EVENTS_ADDRESS").unwrap_or_else(|_| DEFAULT_EVENTS_ADDRESS.to_string()));
pub static CHAIN_NAME: Lazy<String> =
    Lazy::new(|| env::var("CHAIN_NAME").unwrap_or_else(|_| DEFAULT_CHAIN_NAME.to_string()));
pub static TTL: Lazy<String> =
    Lazy::new(|| env::var("TTL").unwrap_or_else(|_| DEFAULT_TTL.to_string()));

pub static COWL_CEP_18_INSTALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CEP_18_INSTALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_CEP_18_INSTALL_PAYMENT_AMOUNT.to_string())
});
pub static COWL_CEP_18_TOKEN_NAME: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CEP_18_TOKEN_NAME").unwrap_or_else(|_| DEFAULT_CEP_18_TOKEN_NAME.to_string())
});
pub static COWL_CEP_18_TOKEN_SYMBOL: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CEP_18_TOKEN_SYMBOL").unwrap_or_else(|_| DEFAULT_CEP_18_TOKEN_SYMBOL.to_string())
});
pub static COWL_CEP_18_COOL_SYMBOL: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CEP_18_COOL_SYMBOL").unwrap_or_else(|_| DEFAULT_CEP_18_COOL_SYMBOL.to_string())
});
pub static COWL_CEP_18_TOKEN_DECIMALS: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CEP_18_TOKEN_DECIMALS")
        .unwrap_or_else(|_| DEFAULT_CEP_18_TOKEN_DECIMALS.to_string())
});

pub static COWL_CEP18_TOKEN_CONTRACT_HASH_NAME: Lazy<String> =
    Lazy::new(|| format!("cep18_contract_hash_{}", *COWL_CEP_18_TOKEN_NAME));
pub static COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME: Lazy<String> =
    Lazy::new(|| format!("cep18_contract_package_hash_{}", *COWL_CEP_18_TOKEN_NAME));

pub static COWL_VESTING_INSTALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_VESTING_INSTALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_VESTING_INSTALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_VESTING_NAME: Lazy<String> = Lazy::new(|| {
    env::var("COWL_VESTING_NAME").unwrap_or_else(|_| DEFAULT_VESTING_NAME.to_string())
});
pub static COWL_VESTING_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_VESTING_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_VESTING_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_TOKEN_TRANSFER_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_TOKEN_TRANSFER_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_TOKEN_TRANSFER_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_DEPOSIT_COWL_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_DEPOSIT_COWL_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_DEPOSIT_COWL_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_DEPOSIT_CSPR_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_DEPOSIT_CSPR_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_DEPOSIT_CSPR_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_CSPR_TO_COWL_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CSPR_TO_COWL_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_CSPR_TO_COWL_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_COWL_TO_CSPR_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_COWL_TO_CSPR_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_COWL_TO_CSPR_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_SET_MODALITIES_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_SET_MODALITIES_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_SET_MODALITIES_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_SWAP_INSTALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_SWAP_INSTALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_SWAP_INSTALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_WITHDRAW_COWL_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_WITHDRAW_COWL_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_WITHDRAW_COWL_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_WITHDRAW_CSPR_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_WITHDRAW_CSPR_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_WITHDRAW_CSPR_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_UPDATE_TIMES_CALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_UPDATE_TIMES_CALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_UPDATE_TIMES_CALL_PAYMENT_AMOUNT.to_string())
});

pub static COWL_SWAP_NAME: Lazy<String> =
    Lazy::new(|| env::var("COWL_SWAP_NAME").unwrap_or_else(|_| DEFAULT_SWAP_NAME.to_string()));

pub const WASM_PATH: &str = "./wasm/";

pub const FUNDED_KEYS_URL: &str =
    "https://raw.githubusercontent.com/casper-network/casper-node-launcher-js/main/src/config.ts";
pub const FUNDED_KEYS_JSON_FILE_PATH: &str = "funded_keys.json";

pub const INSTALLER: &str = "Installer";
pub const USER_1: &str = "User_1";
pub const USER_2: &str = "User_2";
