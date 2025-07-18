# Importing Accounts

You can export your private key from wallet (Ready) and import it into the file holding the accounts info (`~/.starknet_accounts/starknet_open_zeppelin_accounts.json` by default).

> ⚠️ **Warning**
>
> **Never share your private key!**
> Anyone with access to your private key can access your account and funds. You are doing this at your own risk.

## Exporting Your Private Key

This section shows how to export your private key from specific wallets.

### Examples

#### Ready (formerly Argent)

1. Open the Ready app > Settings.
<br/>
<br/>
<img src="./img/ready_export_1.png" width="300"/>

2. Click on the current account.
<br/>
<br/>
<img src="./img/ready_export_2.png" width="300"/>

3. Click on "Export private key".
<br/>
<br/>
<img src="./img/ready_export_3.png" width="300"/>

4. Enter your password.
<br/>
<br/>
<img src="./img/ready_export_4.png" width="300"/>

5. Copy your private key.
<br/>
<br/>
<img src="./img/ready_export_5.png" width="300"/>


#### Braavos

1. Open the Braavos app > Wallet settings.
<br/>
<br/>
<img src="./img/braavos_export_1.png" width="300"/>

2. Click on "Privacy & Security".
<br/>
<br/>
<img src="./img/braavos_export_2.png" width="300"/>

3. Click on "Export private key".
<br/>
<br/>
<img src="./img/braavos_export_3.png" width="300"/>

4. Enter your password.
<br/>
<br/>
<img src="./img/braavos_export_4.png" width="300"/>

5. Copy your private key.
<br/>
<br/>
<img src="./img/braavos_export_5.png" width="300"/>

## Importing an Account

### Examples

#### General Example

To import an account into the file holding the accounts info (`~/.starknet_accounts/starknet_open_zeppelin_accounts.json` by default), use the `account import` command.

```shell
$ sncast \
    account import \
	--network sepolia \
    --name account_123 \
    --address 0x1 \
    --private-key 0x2 \
    --type oz
```
> 📝 **Note** 
> The `--name` can be omitted as this is optional. A default name will be generated for the account.

#### Passing Private Key in an Interactive

If you don't want to pass the private key in the command (because of safety aspect), you can skip `--private-key` flag. You will be prompted to enter the private key in interactive mode.

<!-- { "ignored": true } -->
```shell
$ sncast \
    account import \
	--network sepolia \
    --name account_123 \
    --address 0x1 \
    --type oz
```

<details>
<summary>Output:</summary>

```shell
Type in your private key and press enter: 
```
</details>
<br>

#### Ready (formerly Argent)

To import Ready account, set the `--type` flag to `ready`.

```shell
$ sncast \
    account import \
	--network sepolia \
    --name account_ready \
    --address 0x1 \
    --private-key 0x2 \
    --type ready
```

#### Braavos

To import Braavos account, set the `--type` flag to `braavos`.

```shell
$ sncast \
    account import \
	--network sepolia \
    --name account_braavos \
    --address 0x1 \
    --private-key 0x2 \
    --type braavos
```

#### OpenZeppelin

To import OpenZeppelin account, set the `--type` flag to `oz` or  `open_zeppelin`.

```shell
$ sncast \
    account import \
	--network sepolia \
    --name account_oz \
    --address 0x1 \
    --private-key 0x2 \
    --type oz
```