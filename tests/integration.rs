mod tests_install_vesting {
    use std::sync::Arc;

    use assert_cmd::Command;
    use cowl_cli::utils::constants::COWL_CEP_18_TOKEN_SYMBOL;
    use once_cell::sync::Lazy;
    use tokio::{sync::Mutex, test};

    const BINARY: &str = "cowl_cli";

    static SETUP_DONE: Lazy<Arc<Mutex<Option<bool>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

    // Run the setup (deploy the contract) only if not done yet
    async fn setup() {
        let mut setup_done = SETUP_DONE.lock().await;
        if setup_done.is_none() {
            let mut cmd = Command::cargo_bin("cowl_cli").unwrap();
            cmd.arg("deploy")
                .write_stdin("y\ny\ny\n")
                .assert()
                .success()
                .stdout(predicates::str::contains(
                    "Command executed: Deploy All Contracts",
                ));
            // Mark the setup as done
            *setup_done = Some(true);
        }
    }

    #[test]
    async fn test_list_funded_addresses() {
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let command = "list-addr";
        cmd.arg(command)
            .assert()
            .success()
            .stdout(predicates::str::contains("List Funded Adresses"))
            .stdout(predicates::str::contains("Installer"))
            .stdout(predicates::str::contains("User_1"))
            .stdout(predicates::str::contains("User_2"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains("User_2"));
    }

    #[test]
    async fn test_deploy_all_contracts() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        cmd.arg("deploy")
            .write_stdin("y\ny\ny\n")
            .assert()
            .success() // Ensure the command runs successfully
            .stdout(predicates::str::contains(
                "Command executed: Deploy All Contracts",
            ));
    }

    #[test]
    async fn test_deploy_token_contract() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        cmd.arg("deploy")
            .arg("--token")
            .write_stdin("y\n")
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "Command executed: Deploy Contracts { token: true, vesting: false, swap: false }",
            ));
    }

    #[test]
    async fn test_deploy_vesting_contract() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        cmd.arg("deploy")
            .arg("--vesting")
            .write_stdin("y\n")
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "Command executed: Deploy Contracts { token: false, vesting: true, swap: false }",
            ));
    }

    #[test]
    async fn test_deploy_swap_contract() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        cmd.arg("deploy")
            .arg("--swap")
            .write_stdin("y\n")
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "Command executed: Deploy Contracts { token: false, vesting: false, swap: true }",
            ));
    }

    #[tokio::test]
    async fn test_vesting_info_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let vesting_type = "Community";

        cmd.arg("info")
            .arg("--vesting-type")
            .arg(vesting_type)
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Vesting Info for {}",
                vesting_type
            )))
            .stdout(predicates::str::contains("VestingInfo".to_string()))
            .stdout(predicates::str::contains(format!(
                "vesting_type: {vesting_type}",
            )))
            .stdout(predicates::str::contains(
                "vesting_address_key: Some(Key::Account",
            ));
    }

    #[tokio::test]
    async fn test_vesting_status_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let vesting_type = "Community";

        cmd.arg("status")
            .arg("--vesting-type")
            .arg(vesting_type)
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Vesting Status for {}",
                vesting_type
            )))
            .stdout(predicates::str::contains("VestingStatus".to_string()))
            .stdout(predicates::str::contains(format!(
                "vesting_type: {vesting_type}",
            )))
            .stdout(predicates::str::contains("total_amount"))
            .stdout(predicates::str::contains("vested_amount"));
    }

    #[tokio::test]
    async fn test_balance_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let vesting_type = "Community";

        cmd.arg("balance")
            .arg("--vesting-type")
            .arg(vesting_type)
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "Command executed: Balance for Community".to_string(),
            ))
            .stdout(predicates::str::contains(
                COWL_CEP_18_TOKEN_SYMBOL.to_string(),
            ));
    }

    #[tokio::test]
    async fn test_balance_with_specific_key() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let vesting_type = "Community";
        let key = "016fd7fb5f002d82f3813c76ac83940d4d886035395ddd9be66c9a4a2993b63aaf"; // Community

        // Execute the command with arguments
        cmd.arg("balance")
            .arg("--vesting-type")
            .arg(vesting_type)
            .arg("--key")
            .arg(key)
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "Command executed: Balance for Community".to_string(),
            ))
            .stdout(predicates::str::contains(
                COWL_CEP_18_TOKEN_SYMBOL.to_string(),
            ));
    }

    #[tokio::test]
    async fn test_transfer_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let from = "01fbe77037c317c12af3a6af08d02d9fc6b3a1636237ae48f77b198a9483d94801"; // Liquidity
        let to = "01868e06026ba9c8695f6f3bb10d44782004dbc144ff65017cf484436f9cf7b0f6";
        let amount = "100000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIOeKQNbCmsyZme2t5U7Lulnn2TfdZkiFANeg89Sy7Pzn";
        let confirmation_response = "y\n";

        cmd.arg("transfer")
            .arg("--from")
            .arg(from)
            .arg("--to")
            .arg(to)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Transfer 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(from))
            .stdout(predicates::str::contains(to))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_transfer_command_treasury_recipient() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let from = "01fbe77037c317c12af3a6af08d02d9fc6b3a1636237ae48f77b198a9483d94801"; // Liquidity
        let to = "Treasury";
        let amount = "100000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIOeKQNbCmsyZme2t5U7Lulnn2TfdZkiFANeg89Sy7Pzn";
        let confirmation_response = "y\n";

        cmd.arg("transfer")
            .arg("--from")
            .arg(from)
            .arg("--to")
            .arg(to)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Transfer 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(from))
            .stdout(predicates::str::contains(to))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_transfer_command_treasury_recipient_account_hash() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let from = "01fbe77037c317c12af3a6af08d02d9fc6b3a1636237ae48f77b198a9483d94801"; // Liquidity
        let to = "account-hash-31dfd6356d4be001607bd2d6b163c9b23967873a849a96813781674cf5e4d96b";
        let amount = "100000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIOeKQNbCmsyZme2t5U7Lulnn2TfdZkiFANeg89Sy7Pzn";
        let confirmation_response = "y\n";

        cmd.arg("transfer")
            .arg("--from")
            .arg(from)
            .arg("--to")
            .arg(to)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Transfer 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(from))
            .stdout(predicates::str::contains(to))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_increase_allowance_key_spender() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let owner = "01fbe77037c317c12af3a6af08d02d9fc6b3a1636237ae48f77b198a9483d94801"; // Liquidity
        let spender = "01868e06026ba9c8695f6f3bb10d44782004dbc144ff65017cf484436f9cf7b0f6";
        let amount = "100000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIOeKQNbCmsyZme2t5U7Lulnn2TfdZkiFANeg89Sy7Pzn";
        let confirmation_response = "y\n";

        cmd.arg("increase-allowance")
            .arg("--owner")
            .arg(owner)
            .arg("--spender")
            .arg(spender)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Increase Allowance 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(spender))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains("Increase allowance"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_increase_allowance_treasury_spender() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let owner = "016fd7fb5f002d82f3813c76ac83940d4d886035395ddd9be66c9a4a2993b63aaf"; // Community
        let spender = "Treasury";
        let amount = "100000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIPZsIcOa1F3PpF8SoOjIaJ1qIrsraqj2APBA1pZV0N+R";
        let confirmation_response = "y\n";

        cmd.arg("increase-allowance")
            .arg("--owner")
            .arg(owner)
            .arg("--spender")
            .arg(spender)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Increase Allowance 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains("of Treasury"))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains("Increase allowance"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_decrease_allowance_key_spender() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let owner = "016fd7fb5f002d82f3813c76ac83940d4d886035395ddd9be66c9a4a2993b63aaf"; // Community
        let spender = "01868e06026ba9c8695f6f3bb10d44782004dbc144ff65017cf484436f9cf7b0f6";
        let amount = "50000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIPZsIcOa1F3PpF8SoOjIaJ1qIrsraqj2APBA1pZV0N+R";
        let confirmation_response = "y\n";

        cmd.arg("decrease-allowance")
            .arg("--owner")
            .arg(owner)
            .arg("--spender")
            .arg(spender)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Decrease Allowance 50.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(spender))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains("Decrease allowance"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_decrease_allowance_treasury_spender() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let owner = "016fd7fb5f002d82f3813c76ac83940d4d886035395ddd9be66c9a4a2993b63aaf"; // Community
        let spender = "Treasury";
        let amount = "50000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIPZsIcOa1F3PpF8SoOjIaJ1qIrsraqj2APBA1pZV0N+R";
        let confirmation_response = "y\n";

        cmd.arg("decrease-allowance")
            .arg("--owner")
            .arg(owner)
            .arg("--spender")
            .arg(spender)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Decrease Allowance 50.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains("of Treasury"))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains("Decrease allowance"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_allowance_with_keys() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let owner = "016fd7fb5f002d82f3813c76ac83940d4d886035395ddd9be66c9a4a2993b63aaf"; // Community
        let spender = "01868e06026ba9c8695f6f3bb10d44782004dbc144ff65017cf484436f9cf7b0f6";

        cmd.arg("allowance")
            .arg("--owner")
            .arg(owner)
            .arg("--spender")
            .arg(spender)
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: {} Allowance",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(owner))
            .stdout(predicates::str::contains(spender))
            .stdout(predicates::str::contains("Allowance for account-hash"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_allowance_with_vesting_types() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let owner = "Community";
        let spender = "Treasury";

        cmd.arg("allowance")
            .arg("--owner")
            .arg(owner)
            .arg("--spender")
            .arg(spender)
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: {} Allowance",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(owner))
            .stdout(predicates::str::contains(spender))
            .stdout(predicates::str::contains("Allowance for account-hash"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_transfer_from_key_spender() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let owner = "01fbe77037c317c12af3a6af08d02d9fc6b3a1636237ae48f77b198a9483d94801"; // Liquidity
        let spender = "01868e06026ba9c8695f6f3bb10d44782004dbc144ff65017cf484436f9cf7b0f6";
        let recipient = "01bfe707f56b46172965fd9e557d32582e5daf677b786bc44c5a584a5956962cea";
        let amount = "100000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIOeKQNbCmsyZme2t5U7Lulnn2TfdZkiFANeg89Sy7Pzn";
        let confirmation_response = "y\n";

        cmd.arg("increase-allowance")
            .arg("--owner")
            .arg(owner)
            .arg("--spender")
            .arg(spender)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Increase Allowance 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(spender))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains("Increase allowance"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));

        let mut cmd = Command::cargo_bin(BINARY).unwrap();

        cmd.arg("transfer-from")
            .arg("--operator")
            .arg(spender)
            .arg("--from")
            .arg(owner)
            .arg("--to")
            .arg(recipient)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: TransferFrom 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(spender))
            .stdout(predicates::str::contains(owner))
            .stdout(predicates::str::contains(spender))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains("Transfer"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_transfer_from_key_spender_recipient_as_vesting_type() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let owner = "01fbe77037c317c12af3a6af08d02d9fc6b3a1636237ae48f77b198a9483d94801"; // Liquidity
        let spender = "Treasury";
        let spender_key = "011540c4793aaae429ba1c4234d28f81602f8ea9a6ee2faca0841064b1c00777aa";
        let recipient = "Community";

        let amount = "100000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIOeKQNbCmsyZme2t5U7Lulnn2TfdZkiFANeg89Sy7Pzn";
        let confirmation_response = "y\n";

        cmd.arg("increase-allowance")
            .arg("--owner")
            .arg(owner)
            .arg("--spender")
            .arg(spender)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Increase Allowance 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(spender))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains("Increase allowance"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));

        let mut cmd = Command::cargo_bin(BINARY).unwrap();

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIBYTk4Pc0Q6F3okf21hVWWJoGzQhuY86aRXjwdO1kYBK";

        cmd.arg("transfer-from")
            .arg("--operator")
            .arg(spender_key) // this can be only signed by a key, not a vesting type
            .arg("--from")
            .arg(owner)
            .arg("--to")
            .arg(recipient)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: TransferFrom 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(owner))
            .stdout(predicates::str::contains(spender_key))
            .stdout(predicates::str::contains(recipient))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("CSPR"))
            .stdout(predicates::str::contains("Transfer"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }
}

mod tests_swap {
    use assert_cmd::Command;
    use chrono::Utc;
    use cowl_cli::utils::constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL};
    use once_cell::sync::Lazy;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    const BINARY: &str = "cowl_cli";

    static SETUP_DONE: Lazy<Arc<Mutex<Option<bool>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

    // Run the setup (deploy the contract) only if not done yet
    async fn setup() {
        let mut setup_done = SETUP_DONE.lock().await;
        if setup_done.is_none() {
            let mut cmd = Command::cargo_bin("cowl_cli").unwrap();
            cmd.arg("deploy")
                .write_stdin("y\ny\ny\n")
                .assert()
                .success()
                .stdout(predicates::str::contains(
                    "Command executed: Deploy All Contracts",
                ));
            // Mark the setup as done
            *setup_done = Some(true);
        }
    }

    #[tokio::test]
    async fn test_desposit_cowl_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let from = "01fbe77037c317c12af3a6af08d02d9fc6b3a1636237ae48f77b198a9483d94801"; // Liquidity
        let amount = "100000000000";

        let base64_key = "MC4CAQAwBQYDK2VwBCIEIOeKQNbCmsyZme2t5U7Lulnn2TfdZkiFANeg89Sy7Pzn";
        let confirmation_response = "y\n";

        cmd.arg("deposit-cowl")
            .arg("--from")
            .arg(from)
            .arg("--amount")
            .arg(amount)
            .write_stdin(format!("{base64_key}\n{confirmation_response}"))
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Deposit 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains(from))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("Balance for 01"))
            .stdout(predicates::str::contains("Balance for Swap"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_desposit_cspr_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let amount = "100000000000";

        let confirmation_response = "y\n";

        cmd.arg("deposit-cspr")
            .arg("--amount")
            .arg(amount)
            .write_stdin(confirmation_response.to_string())
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "Command executed: Deposit 100.00 CSPR".to_string(),
            ))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("Balance for 01"))
            .stdout(predicates::str::contains("Balance for Swap"))
            .stdout(predicates::str::contains(" motes"));
    }

    #[tokio::test]
    async fn test_withdraw_cowl_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let amount = "100000000000";

        let confirmation_response = "y\n";

        cmd.arg("withdraw-cowl")
            .arg("--amount")
            .arg(amount)
            .write_stdin(confirmation_response.to_string())
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "Command executed: Withdraw 100.00 {}",
                COWL_CEP_18_TOKEN_SYMBOL.clone()
            )))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("Balance for 01"))
            .stdout(predicates::str::contains("Balance for Swap"))
            .stdout(predicates::str::contains(COWL_CEP_18_TOKEN_SYMBOL.clone()));
    }

    #[tokio::test]
    async fn test_withdraw_cspr_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let amount = "100000000000";

        let confirmation_response = "y\n";

        cmd.arg("withdraw-cspr")
            .arg("--amount")
            .arg(amount)
            .write_stdin(confirmation_response.to_string())
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "Command executed: Withdraw 100.00 CSPR".to_string(),
            ))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("Balance for 01"))
            .stdout(predicates::str::contains("Balance for Swap"))
            .stdout(predicates::str::contains(" motes"));
    }

    #[tokio::test]
    async fn test_swap_balance_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();

        cmd.arg("swap-balance")
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "Command executed: Get Swap contract balance".to_string(),
            ))
            .stdout(predicates::str::contains(
                COWL_CEP_18_TOKEN_SYMBOL.to_string(),
            ))
            .stdout(predicates::str::contains(
                COWL_CEP_18_COOL_SYMBOL.to_string(),
            ));
    }

    #[tokio::test]
    async fn test_update_times_command() {
        setup().await;
        let mut cmd = Command::cargo_bin(BINARY).unwrap();
        let start_time = (Utc::now().timestamp() as u64).to_string();
        let duration = 3600u64.to_string();

        let confirmation_response = "y\n";

        cmd.arg("update-times")
            .arg("--start-time")
            .arg(&start_time)
            .arg("--duration")
            .arg(&duration)
            .write_stdin(confirmation_response.to_string())
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "Command executed: Update times start: ".to_string(),
            ))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Wait deploy_hash"))
            .stdout(predicates::str::contains("Processed deploy hash"))
            .stdout(predicates::str::contains("START:"))
            .stdout(predicates::str::contains("END:"))
            .stdout(predicates::str::contains("_u64"))
            .stdout(predicates::str::contains("UTC"));
    }
}
