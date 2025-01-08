use crate::{
    commands,
    utils::{
        config::get_key_pair_from_vesting,
        constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL},
        format_with_thousands_separator,
    },
};
use casper_rust_wasm_sdk::{
    helpers::motes_to_cspr,
    types::{key::Key, public_key::PublicKey},
};
use clap::{Parser, Subcommand};
use cowl_vesting::enums::VestingType;
use std::fmt::{self, Display};
use strum::IntoEnumIterator;

/// CLI Tool for managing contracts and token distributions
#[derive(Parser)]
#[command(name = "cowl-cli")]
#[command(version = "1.0")]
#[command(about = "A CLI for Cowl token vesting admin", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List funded addresses stored in the configuration.
    #[command(
        name = "list-types",
        about = "List all vesting types from current config"
    )]
    Types,

    /// List funded addresses stored in the configuration.
    #[command(
        name = "list-addr",
        about = "List all funded addresses from current config"
    )]
    ListFundedAdresses,

    /// Deploy smart contracts.
    #[command(
        name = "deploy",
        about = "Deploy contracts for the token or vesting or both by default"
    )]
    DeployContracts {
        /// Deploy only the token contract.
        #[arg(long, help = "Deploy only the token contract")]
        token: bool,

        /// Deploy only the vesting contract.
        #[arg(long, help = "Deploy only the vesting contract")]
        vesting: bool,

        /// Deploy only the swap contract.
        #[arg(long, help = "Deploy only the swap contract")]
        swap: bool,
    },

    /// Retrieve vesting information.
    #[command(
        name = "info",
        about = "Get static vesting allocation information about a vesting type"
    )]
    VestingInfo {
        /// Specify the vesting type (e.g., linear, cliff).
        #[arg(long, help = "The vesting type to retrieve information for")]
        vesting_type: String,

        /// Call the entry point in the contract for more detailed information.
        #[arg(long, help = "Call the contract's entry point before")]
        call_entry_point: bool,
    },

    /// Check the status of a vesting type.
    #[command(name = "status", about = "Check the current status of a vesting type")]
    VestingStatus {
        /// Specify the vesting type (e.g., linear, cliff).
        #[arg(
            long,
            help = "The vesting type to check the status for. Contract's entrypoint will be called before to update the value to retrieve."
        )]
        vesting_type: String,
    },

    /// Retrieve the balance of a vesting or public key.
    #[command(
        name = "balance",
        about = "Retrieve the balance for a specific vesting type or key (Public key or Account hash)"
    )]
    Balance {
        /// Specify the vesting type (optional).
        #[arg(long, help = "The vesting type to retrieve the balance for")]
        vesting_type: Option<String>,

        /// Specify the public or account key (optional).
        #[arg(
            long,
            help = "The public key or account hash to retrieve the balance for"
        )]
        key: Option<String>,
    },

    /// Transfer tokens between accounts or vesting types.
    #[command(
        name = "transfer",
        about = "Transfers tokens between a source and a destination account or vesting type"
    )]
    Transfer {
        /// Specify the source (public key signing).
        #[arg(
            long,
            help = "The source (public key signing) to transfer from.
            Example: 016fd7fb5f002d82f3813c76ac83940d4d886035395ddd9be66c9a4a2993b63aaf"
        )]
        from: String,

        /// Specify the destination (public key, account hash, or vesting type).
        #[arg(
            long,
            help = "The destination (public key, account hash, or vesting type) to transfer to.
            Example: 01868e06026ba9c8695f6f3bb10d44782004dbc144ff65017cf484436f9cf7b0f6
            Example: Treasury
            Example: account-hash-31dfd6356d4be001607bd2d6b163c9b23967873a849a96813781674cf5e4d96b"
        )]
        to: String,

        /// The amount to transfer.
        #[arg(
            long,
            help = "The amount to transfer in the smallest unit (e.g., '100000000000' represents 100 COWL). Example: '100000000000'"
        )]
        amount: String,
    },

    /// Allowance command that checks or manages allowances between accounts.
    #[command(
        name = "allowance",
        about = "Checks or manages the allowance between the owner and spender"
    )]
    Allowance {
        #[arg(
            long,
            help = "The owner (vesting type or public key/account hash) of the allowance"
        )]
        owner: String,

        /// Specify the spender (vesting type or public key).
        #[arg(
            long,
            help = "The spender (vesting type or public key/account hash) of the allowance"
        )]
        spender: String,
    },

    /// Transfer tokens from an account using a previously granted allowance.
    #[command(
        name = "transfer-from",
        about = "Transfers tokens from an account using a previously granted allowance"
    )]
    TransferFrom {
        /// Specify the allowed operator (public key signing).
        #[arg(
            long,
            help = "The allowed operator (public key signing) to transfer from"
        )]
        operator: String,

        /// Specify the source (vesting type or public key/account hash) to transfer from.
        #[arg(
            long,
            help = "The source (vesting type or public key/account hash) to transfer from"
        )]
        from: String,

        /// Specify the destination (vesting type or public key/account hash).
        #[arg(
            long,
            help = "The destination (vesting type or public key/account hash) to transfer to"
        )]
        to: String,

        /// The amount to transfer.
        #[arg(
            long,
            help = "The amount to transfer in the smallest unit (e.g., '100000000000' represents 100 COWL). Example: '100000000000'"
        )]
        amount: String,
    },

    /// Increase the allowance of a spender for an owner.
    #[command(
        name = "increase-allowance",
        about = "Increases the allowance of a spender for a given owner"
    )]
    IncreaseAllowance {
        #[arg(long, help = "The owner (public key signing) of the tokens")]
        owner: String,

        #[arg(
            long,
            help = "The spender (vesting type or public key/account hash) to increase allowance of"
        )]
        spender: String,

        /// The amount to increase.
        #[arg(
            long,
            help = "The amount to increase in the smallest unit (e.g., '100000000000' represents 100 COWL). Example: '100000000000'"
        )]
        amount: String,
    },

    /// Decrease the allowance of a spender for an owner.
    #[command(
        name = "decrease-allowance",
        about = "Decreases the allowance of a spender for a given owner"
    )]
    DecreaseAllowance {
        #[arg(long, help = "The owner (public key signing) of the tokens")]
        owner: String,

        #[arg(
            long,
            help = "The spender (vesting type or public key/account hash) to decrease allowance of"
        )]
        spender: String,

        /// The amount to decrease.
        #[arg(
            long,
            help = "The amount to decrease in the smallest unit (e.g., '100000000000' represents 100 COWL). Example: '100000000000'"
        )]
        amount: String,
    },

    /// Fund and retrieve the balance of a vesting or public key.
    #[command(
        name = "fund-cspr",
        about = "Fund and retrieve the balance for a specific vesting type or key (Public key or Account hash)"
    )]
    Fund {
        /// Specify the vesting type (optional).
        #[arg(long, help = "The vesting type to retrieve the balance for")]
        vesting_type: Option<String>,

        /// Specify the public or account key (optional).
        #[arg(
            long,
            help = "The public key or account hash to retrieve the balance for"
        )]
        key: Option<String>,
        /// The amount to fund.
        #[arg(
            long,
            help = "The amount to fund in the smallest unit (e.g., '2500000000' represents 2.5 CSPR). Example: '2500000000' (minimum)"
        )]
        amount: String,
    },

    /// Upgrade contratc and enable events
    #[command(
        name = "upgrade-events",
        about = "Upgrade and enable evensts (specific upgrade 01/2025)"
    )]
    UpgradeEvents,

    /// Upgrade contratc and enable events
    #[command(name = "deposit-cowl", about = "Deposit COWL")]
    DepositCowl {
        /// Specify the source (public key signing).
        #[arg(
            long,
            help = "The source (public key signing) to deposit from.
                    Example: 016fd7fb5f002d82f3813c76ac83940d4d886035395ddd9be66c9a4a2993b63aaf"
        )]
        from: String,

        /// The amount to deposit.
        #[arg(
            long,
            help = "The amount to deposit in the smallest unit (e.g., '100000000000' represents 100 COWL). Example: '100000000000'"
        )]
        amount: String,
    },
    #[command(name = "deposit-cspr", about = "Deposit CSPR")]
    DepositCspr {
        /// The amount to deposit.
        #[arg(
            long,
            help = "The amount to deposit in the smallest unit (e.g., '100000000000' represents 100 CSPR). Example: '100000000000'"
        )]
        amount: String,
    },
    #[command(name = "withdraw-cowl", about = "Withdraw COWL")]
    WithdrawCowl {
        /// The amount to withdraw.
        #[arg(
            long,
            help = "The amount to withdraw in the smallest unit (e.g., '100000000000' represents 100 COWL). Example: '100000000000'"
        )]
        amount: String,
    },
    #[command(name = "withdraw-cspr", about = "Withdraw COWL")]
    WithdrawCspr {
        /// The amount to withdraw.
        #[arg(
            long,
            help = "The amount to withdraw in the smallest unit (e.g., '100000000000' represents 100 CSPR). Example: '100000000000'"
        )]
        amount: String,
    },
}

pub async fn run() {
    let cli = Cli::parse();

    log::info!("Command executed: {}", cli.command);

    match cli.command {
        Commands::Types => {
            for vesting_type in VestingType::iter() {
                println!("{vesting_type}");
            }
        }
        Commands::ListFundedAdresses => commands::addresses::print_funded_addresses().await,
        Commands::DeployContracts {
            token,
            vesting,
            swap,
        } => {
            let result = if token {
                commands::deploy::deploy_cep18_token().await
            } else if vesting {
                commands::deploy::deploy_vesting_contract().await
            } else if swap {
                commands::deploy::deploy_swap_contract().await
            } else {
                commands::deploy::deploy_all_contracts().await
            };

            if let Err(e) = result {
                log::error!("Error deploying contracts: {}", e);
                std::process::exit(1);
            }
        }
        Commands::VestingInfo {
            vesting_type,
            call_entry_point,
        } => {
            commands::info::print_vesting_info(
                vesting_type
                    .as_str()
                    .try_into()
                    .expect("Failed to convert vesting type"),
                call_entry_point,
            )
            .await
        }
        Commands::VestingStatus { vesting_type } => {
            let call_entry_point = true; // Always call entry point before getting status
            commands::status::print_vesting_status(
                vesting_type
                    .as_str()
                    .try_into()
                    .expect("Failed to convert vesting type"),
                call_entry_point,
            )
            .await
        }
        Commands::Balance { vesting_type, key } => {
            commands::balance::print_balance(
                vesting_type.map(|f| {
                    f.as_str()
                        .try_into()
                        .expect("Failed to convert vesting type")
                }),
                key.map(|formatted_str| parse_key_from_formatted_str(&formatted_str)),
            )
            .await
        }
        Commands::Transfer { from, to, amount } => {
            // Retrieve the key pair for the recipient
            let to_key = if let Some(key_pair) = get_key_pair_from_vesting(&to).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&to)
            };

            commands::transfer::print_transfer(
                PublicKey::new(&from).expect("Failed to convert public key to key"),
                to_key,
                amount,
            )
            .await
        }
        Commands::Allowance { owner, spender } => {
            // Retrieve the key pair for the owner
            let owner_key = if let Some(key_pair) = get_key_pair_from_vesting(&owner).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&owner)
            };

            // Retrieve the key pair for the spender
            let spender_key = if let Some(key_pair) = get_key_pair_from_vesting(&spender).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&spender)
            };

            commands::allowance::print_get_allowance(&owner_key, &spender_key).await
        }
        Commands::TransferFrom {
            operator,
            from,
            to,
            amount,
        } => {
            // Retrieve the key pair for the recipient
            let from_key = if let Some(key_pair) = get_key_pair_from_vesting(&from).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&from)
            };

            // Retrieve the key pair for the recipient
            let to_key = if let Some(key_pair) = get_key_pair_from_vesting(&to).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&to)
            };

            commands::transfer_from::print_transfer_from(
                PublicKey::new(&operator).expect("Failed to convert public key to key"),
                from_key,
                to_key,
                amount,
            )
            .await
        }
        Commands::IncreaseAllowance {
            owner,
            spender,
            amount,
        } => {
            // Retrieve the key pair for the spender
            let spender_key = if let Some(key_pair) = get_key_pair_from_vesting(&spender).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&spender)
            };

            commands::allowance::print_increase_allowance(
                &PublicKey::new(&owner).expect("Failed to convert public key to key"),
                &spender_key,
                amount,
            )
            .await
        }
        Commands::DecreaseAllowance {
            owner,
            spender,
            amount,
        } => {
            // Retrieve the key pair for the spender
            let spender_key = if let Some(key_pair) = get_key_pair_from_vesting(&spender).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&spender)
            };
            commands::allowance::print_decrease_allowance(
                &PublicKey::new(&owner).expect("Failed to convert public key to key"),
                &spender_key,
                amount,
            )
            .await
        }
        Commands::Fund {
            vesting_type,
            key,
            amount,
        } => {
            commands::fund::print_fund_addresses(
                vesting_type.map(|f| {
                    f.as_str()
                        .try_into()
                        .expect("Failed to convert vesting type")
                }),
                key.map(|formatted_str| parse_key_from_formatted_str(&formatted_str)),
                amount,
            )
            .await
        }
        Commands::UpgradeEvents => commands::upgrade_events::print_upgrade_events().await,
        Commands::DepositCowl { from, amount } => {
            commands::deposit_cowl::print_deposit_cowl(
                PublicKey::new(&from).expect("Failed to convert public key to key"),
                amount,
            )
            .await
        }
        Commands::DepositCspr { amount } => {
            commands::deposit_cspr::print_deposit_cspr(amount).await
        }
        Commands::WithdrawCowl { amount } => {
            commands::withdraw_cowl::print_withdraw_cowl(amount).await
        }
        Commands::WithdrawCspr { amount } => {
            commands::withdraw_cspr::print_withdraw_cspr(amount).await
        }
    }
}

impl Display for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Commands::Types => write!(f, "List vesting Types"),
            Commands::ListFundedAdresses => write!(f, "List Funded Adresses"),

            Commands::DeployContracts {
                token,
                vesting,
                swap,
            } => {
                if *token || *vesting || *swap {
                    write!(
                        f,
                        "Deploy Contracts {{ token: {}, vesting: {}, swap: {} }}",
                        token, vesting, swap
                    )
                } else {
                    write!(
                        f,
                        "Deploy All Contracts {{ token: true, vesting: true, swap: true }}"
                    )
                }
            }

            Commands::VestingInfo {
                vesting_type,
                call_entry_point: _,
            } => write!(f, "Vesting Info for {vesting_type}",),
            Commands::VestingStatus { vesting_type } => {
                write!(f, "Vesting Status for {vesting_type}",)
            }
            Commands::Balance { vesting_type, key } => {
                if let Some(vesting_type) = vesting_type {
                    write!(f, "Balance for {}", vesting_type)
                } else if let Some(key) = key {
                    write!(f, "Balance for {}", key)
                } else {
                    write!(f, "Balance: No vesting_type or key provided",)
                }
            }
            Commands::Transfer { from, to, amount } => {
                write!(
                    f,
                    "Transfer {} {} ({} {}) \nfrom {} \nto: {}",
                    format_with_thousands_separator(&motes_to_cspr(amount).unwrap()),
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    amount,
                    *COWL_CEP_18_COOL_SYMBOL,
                    from.clone(),
                    to.clone()
                )
            }
            Commands::Allowance { owner, spender } => {
                write!(
                    f,
                    "{} Allowance \nfrom {owner} \nto {spender}",
                    *COWL_CEP_18_TOKEN_SYMBOL
                )
            }
            Commands::TransferFrom {
                operator,
                from,
                to,
                amount,
            } => {
                write!(
                    f,
                    "TransferFrom {} {} ({} {}) \nby {} \nfrom {} \nto: {}",
                    format_with_thousands_separator(&motes_to_cspr(amount).unwrap()),
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    amount,
                    *COWL_CEP_18_COOL_SYMBOL,
                    operator.clone(),
                    from.clone(),
                    to.clone()
                )
            }
            Commands::IncreaseAllowance {
                owner: _,
                spender,
                amount,
            } => {
                write!(
                    f,
                    "Increase Allowance {} {} ({} {}) \nof {}",
                    format_with_thousands_separator(&motes_to_cspr(amount).unwrap()),
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    amount,
                    *COWL_CEP_18_COOL_SYMBOL,
                    spender.clone(),
                )
            }
            Commands::DecreaseAllowance {
                owner: _,
                spender,
                amount,
            } => {
                write!(
                    f,
                    "Decrease Allowance {} {} ({} {}) \nof {}",
                    format_with_thousands_separator(&motes_to_cspr(amount).unwrap()),
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    amount,
                    *COWL_CEP_18_COOL_SYMBOL,
                    spender.clone(),
                )
            }
            Commands::Fund {
                vesting_type,
                key,
                amount,
            } => {
                let entity = vesting_type
                    .clone()
                    .map(|v| v.to_string())
                    .or_else(|| key.clone());

                let message = match entity {
                    Some(ref entity_str) => format!(
                        "CSPR Funding of {} CSPR ({} motes) for {}",
                        format_with_thousands_separator(&motes_to_cspr(amount).unwrap()),
                        amount,
                        entity_str
                    ),
                    None => "CSPR Funding: No vesting_type or key provided".to_string(),
                };

                write!(f, "{}", message)
            }
            Commands::UpgradeEvents => write!(f, "Upgrade and enable Events"),
            Commands::DepositCowl { from, amount } => write!(
                f,
                "Deposit {} {} ({} {}) \nfrom {}",
                format_with_thousands_separator(&motes_to_cspr(amount).unwrap()),
                *COWL_CEP_18_TOKEN_SYMBOL,
                amount,
                *COWL_CEP_18_COOL_SYMBOL,
                from.clone(),
            ),
            Commands::DepositCspr { amount } => write!(
                f,
                "Deposit {} CSPR ({} motes) \nfrom Installer",
                format_with_thousands_separator(&motes_to_cspr(amount).unwrap()),
                amount,
            ),
            Commands::WithdrawCowl { amount } => write!(
                f,
                "Withdraw {} {} ({} {}) \nto Installer",
                format_with_thousands_separator(&motes_to_cspr(amount).unwrap()),
                *COWL_CEP_18_TOKEN_SYMBOL,
                amount,
                *COWL_CEP_18_COOL_SYMBOL,
            ),
            Commands::WithdrawCspr { amount } => write!(
                f,
                "Withdraw {} CSPR ({} motes) \nto Installer",
                format_with_thousands_separator(&motes_to_cspr(amount).unwrap()),
                amount,
            ),
        }
    }
}

fn parse_key_from_formatted_str(formatted_str: &str) -> Key {
    if let Ok(public_key) = PublicKey::new(formatted_str) {
        Key::from_formatted_str(&public_key.to_account_hash().to_formatted_string())
            .expect("Failed to convert public key to key")
    } else {
        Key::from_formatted_str(formatted_str).expect("Failed to convert key")
    }
}
