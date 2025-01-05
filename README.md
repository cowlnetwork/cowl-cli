# Cowl-CLI

Cowl-CLI is a Command Line Interface (CLI) tool designed for managing smart contracts and token distribution within the Cowl project. It allows users to deploy contracts, manage token allocations, check balances, and much more.

## Table of Contents

1. [Installation](#installation)
2. [Usage](#usage)
3. [Available Commands](#available-commands)
   - [List Vesting Types](#list-vesting-types)
   - [List Funded Addresses](#list-funded-addresses)
   - [Deploy Contracts](#deploy-contracts)
   - [Get Vesting Info](#get-vesting-info)
   - [Check Vesting Status](#check-vesting-status)
   - [Check Balance](#check-balance)
   - [Transfer Tokens](#transfer-tokens)
   - [Manage Allowances](#manage-allowances)
   - [Other Commands](#other-commands)

---

## Installation

1. Clone this repository:
   ```bash
   git clone https://github.com/your-repo/cowl-cli.git
   ```
2. Navigate to the project directory:
   ```bash
   cd cowl-cli
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```
4. Run the CLI from the generated binary:

   ```bash
   ./target/release/cowl-cli
   ```

5. Alternatively, you can run the project directly without building:
   ```bash
   cargo run
   ```

---

## Configuration

By default, the configuration is based on [casper-node-launcher-js](https://github.com/casper-network/casper-node-launcher-js), but it can be overridden by the `.env` file.

1. Copy the contents of `.env.example` to a new `.env` file:

   ```bash
   cp .env.example .env
   ```

2. Modify the `.env` file to configure the necessary variables you want to overload:

   ```bash
   \# RPC_ADDRESS=http://localhost:7777
   \# EVENTS_ADDRESS=http://localhost:9999/events/main
   \# CHAIN_NAME=casper-net-1
   \#
   \# PRIVATE_KEY_INSTALLER = MC4CAQAwBQYDK2VwBCIEII8ULlk1CJ12ZQ+bScjBt/IxMAZNggClWqK56D1/7CbI
   \# PATH_PRIVATE_KEY_INSTALLER = /opt2/casper/casper-nctl-2-docker/assets/users/user-1/secret_key.pem
   \# PRIVATE_KEY_USER_1 = MC4CAQAwBQYDK2VwBCIEII8ULlk1CJ12ZQ+bScjBt/IxMAZNggClWqK56D1/7CbI
   \# PRIVATE_KEY_USER_2 = MC4CAQAwBQYDK2VwBCIEII8ULlk1CJ12ZQ+bScjBt/IxMAZNggClWqK56D1/7CbY
   \# PUBLIC_KEY_TREASURY = 01868e06826ba9c8695f6f3bb10d44782004dbc144ff65017cf484436f9cf7b0f6
   ```

3. Be sure to fill in the correct values for each variable. By default, the configuration for the keys will be provided by [this file](https://raw.githubusercontent.com/casper-network/casper-node-launcher-js/main/src/config.ts).

## Usage

All commands follow this basic structure:

```bash
cowl-cli <command> [options]
```

To display help for a specific command, use:

```bash
cowl-cli <command> --help
```

---

## Available Commands

### List Vesting Types

Displays all available vesting types in the current configuration.

```bash
cowl-cli list-types
```

### List Funded Addresses

Shows all funded addresses in the configuration.

```bash
cowl-cli list-addr
```

### Deploy Contracts

Deploy token contracts, vesting contracts, or both (default).

```bash
cowl-cli deploy [--token] [--vesting]
```

- `--token`: Deploys only the token contract.
- `--vesting`: Deploys only the vesting contract.

### Get Vesting Info

Retrieve details about a specific vesting type.

```bash
cowl-cli info --vesting-type <type> [--call-entry-point]
```

- `--vesting-type`: Specify the vesting type (e.g., `linear`, `cliff`).
- `--call-entry-point`: Enables fetching information directly through the contract's entry point.

### Check Vesting Status

Check the current status of a vesting type by calling the contract's entry point.

```bash
cowl-cli status --vesting-type <type>
```

### Check Balance

Retrieve the balance of a vesting type or a public key.

```bash
cowl-cli balance [--vesting-type <type>] [--key <key>]
```

- `--vesting-type`: The vesting type.
- `--key`: Public key or account hash.

### Transfer Tokens

Transfer tokens between accounts or vesting types.

```bash
cowl-cli transfer --from <source> --to <destination> --amount <amount>
```

- `--from`: Source (signing public key).
- `--to`: Destination (public key, account hash, or vesting type).
- `--amount`: Amount to transfer in minimal units.

### Manage Allowances

#### Check an Allowance

```bash
cowl-cli allowance --owner <owner> --spender <spender>
```

- `--owner`: The owner of the funds.
- `--spender`: The beneficiary of the allowance.

#### Increase an Allowance

```bash
cowl-cli increase-allowance --owner <owner> --spender <spender> --amount <amount>
```

- `--owner`: The owner of the funds.
- `--spender`: The beneficiary of the allowance.
- `--amount`: The amount to add to the current allowance.

#### Decrease an Allowance

```bash
cowl-cli decrease-allowance --owner <owner> --spender <spender> --amount <amount>
```

- `--owner`: The owner of the funds.
- `--spender`: The beneficiary of the allowance.
- `--amount`: The amount to subtract from the current allowance.

---

## Other Commands

Additional commands may be added based on the evolving needs of the project. Use `cowl-cli --help` to explore the latest options.

---

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.

---

## Contributing

Contributions are welcome! Please submit a pull request or open an issue to discuss improvements or bug fixes.
