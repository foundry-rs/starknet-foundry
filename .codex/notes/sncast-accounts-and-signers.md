# sncast accounts, signers, and legacy keystores

This note records the mental model discovered from the current `sncast` implementation. It is a baseline for future account/signer work.

## Core concepts

Four distinct concepts are currently intertwined:

| Concept | Meaning |
| --- | --- |
| `AccountType` | Account contract implementation: OpenZeppelin, Ready (formerly Argent), or Braavos. Orthogonal to signer choice. |
| `AccountData` | Persisted account metadata: public key, address, salt, deployment status, class hash, execution-encoding hint, account type, and signer information. |
| `SignerType` | Actual signer attached to an accounts-file entry: `Local { private_key }` or `Ledger { ledger_path }`. |
| `SignerSource` | Routing/persistence choice: native accounts file, Ledger, or legacy starkli keystore. It is not itself the runtime signer. |

Key definitions:

- `crates/sncast/src/lib.rs`: `AccountType`, `AccountData`, account loading/building.
- `crates/sncast/src/helpers/signer.rs`: `SignerType`, `SignerSource`, `AccountVariant`, `with_account!`.

`SignerType` is an untagged enum flattened into `AccountData` for backward compatibility. Native JSON therefore contains either a top-level `private_key` or `ledger_path`, without an explicit signer tag.

## Account/signer modes

| Mode | User locator | Metadata | Secret/signing material | Runtime account |
| --- | --- | --- | --- | --- |
| Native local | `--account <name>` | Network-keyed accounts JSON | Plaintext `private_key` in the entry | `SingleOwnerAccount<LocalWallet>` |
| Native Ledger | `--account <name>` | Same accounts JSON | Only `ledger_path` is stored; secret remains on device | `SingleOwnerAccount<LedgerSigner>` |
| Legacy keystore | `--keystore key.json --account account.json` | Separate starkli account JSON | Encrypted starkli/Web3 keystore | Decrypted into `LocalWallet` |
| Built-in devnet | `--account devnet-N` | Fetched from devnet API | Private key returned by devnet | `SingleOwnerAccount<LocalWallet>` |

The important conclusion is that keystore is not a third runtime signer beside local and Ledger. It is a compatibility storage/input protocol which materializes a local signer.

## Native accounts file

Default path:

```text
~/.starknet_accounts/starknet_open_zeppelin_accounts.json
```

Shape:

```json
{
  "alpha-sepolia": {
    "account-name": {
      "public_key": "0x...",
      "address": "0x...",
      "type": "open_zeppelin",
      "private_key": "0x..."
    }
  }
}
```

A Ledger entry replaces `private_key` with `ledger_path`. Network selection comes from the provider chain ID; the account name is resolved within that network.

### Lifecycle

- `account create`
  - Selects OZ, Ready, or Braavos factory.
  - Uses a supplied local key, a generated random key, or a Ledger path.
  - Derives the public key and address, checks the class, estimates deployment cost, and writes an undeployed entry.
- `account import`
  - Accepts `--private-key`, `--private-key-file`, hidden interactive input, or a Ledger locator.
  - Detects deployment and class hash from the network.
  - If salt is provided, recomputes and validates the address.
  - Always writes the native accounts file; this is not how an existing starkli keystore pair is imported.
- `account deploy`
  - Native mode selects the entry with subcommand flag `--name`, not global `--account`.
  - Requires stored `type`, `class_hash`, and `salt`.
  - Uses an account factory parameterized by local or Ledger signer.
  - Marks `deployed: true` after transaction submission.
- `account list`
  - Hides local private keys by default.
  - Always shows Ledger paths.
  - `--display-private-keys` exposes local keys.
- `account delete`
  - Deletes only local metadata, scoped to a network; it does not alter the on-chain account.

Automatic `account-N` generation scans all networks. Account uniqueness when writing is enforced per network.

## Ledger

Account create/import accepts:

```text
--ledger-path <EIP-2645 path>
--ledger-account-id N
```

Account ID `N` expands to:

```text
m//starknet'/sncast'/0'/N'/0
```

The parser:

- requires the EIP-2645 six-level structure;
- supports `m//` as shorthand for `m/2645'/`;
- accepts text hash segments such as `starknet'` and `sncast'`;
- enforces hardened path levels where required;
- stores/displays the resolved canonical numeric path.

After create/import, the path is bound to the saved account. Subsequent declare, invoke, deploy, and multicall commands need only `--account <name>`.

Ledger account deployment explicitly compares the device public key with the stored public key. Ordinary deployed-account construction does not perform that comparison before returning the runtime account.

The independent `sncast ledger` namespace provides `app-version`, `get-public-key`, and `sign-hash`; these commands do not require a saved account.

## Legacy starkli keystore compatibility

Keystore mode uses two files:

```text
--keystore key.json
--account account.json
```

In this mode, global `--account` changes meaning from a native account name to the path of a starkli account JSON file.

- The keystore holds the encrypted private key.
- The account JSON holds variant and deployment metadata.
- Loading decrypts with `KEYSTORE_PASSWORD`, or prompts.
- The parsed result becomes `AccountData` with `SignerType::Local`.

Supported account JSON variants:

- OpenZeppelin: public key at `/variant/public_key`.
- Ready: owner at `/variant/owner`; legacy type name `argent` maps to `ready`.
- Braavos: requires multisig off and exactly one seed signer.

`account create` in keystore mode:

- generates or accepts a local private key;
- encrypts it using `CREATE_KEYSTORE_PASSWORD`, or prompts;
- writes a starkli-compatible account JSON;
- supports OZ, Ready, and Braavos in code;
- does not write the native accounts file.

After deployment, sncast updates the starkli account document by setting status to `deployed`, adding the computed address, and removing salt and Braavos deployment context.

## Runtime transaction path

`get_account` chooses native accounts file, keystore pair, or built-in devnet, then returns:

```rust
AccountVariant::LocalWallet(SingleOwnerAccount<..., LocalWallet>)
AccountVariant::Ledger(SingleOwnerAccount<..., LedgerSigner<...>>)
```

Because these are different generic Rust types, transaction call sites dispatch through `with_account!`.

This runtime path is used by:

- `declare`;
- `declare-from`;
- contract `deploy`, including automatic declaration;
- `invoke`;
- `multicall run`;
- `multicall execute`.

Read-only `call` does not require an account. `get balance` normally reads only the saved address, but keystore and devnet modes construct the runtime account to obtain it.

Before returning a deployed runtime account, sncast:

- requires a stored address;
- checks the address exists on the current chain using the pre-confirmed nonce;
- selects legacy/new execution encoding from `legacy`, or detects it from the account class;
- sets the runtime account block to `pre_confirmed`.

Account deployment is separate: it uses generic account factories directly instead of `AccountVariant`.

## Configuration

`account`, `accounts-file`, and `keystore` can come from CLI, local config, or global config. Precedence is:

```text
CLI
local named profile
local default
global named profile
global default
internal defaults
```

Profiles for native local/Ledger accounts store account name plus accounts-file. Keystore profiles store starkli account-file path plus keystore path.

Current behavior worth retaining in mind:

- `--accounts-file` and `--keystore` are not mutually exclusive at the CLI level.
- Keystore normally takes precedence during runtime loading.
- `show-config` hides the accounts-file path when keystore is active.
- Ledger plus keystore is rejected through `SignerSource` in account creation.

## Built-in devnet accounts

Names starting with `devnet-` form an implicit namespace. `devnet-N` obtains the Nth predeployed account, including its private key, from the devnet API and constructs a local wallet without requiring the default accounts file.

If the same `devnet-N` name exists in the selected accounts file, the saved entry wins and sncast prints a warning. A custom explicitly selected accounts file must still exist.

## Architectural pressure points

- `SignerType` is the real persisted signer discriminant.
- `SignerSource` mixes signer selection with persistence/backend selection.
- Keystore is a compatibility backend, not a distinct runtime signer.
- `get_account_from_accounts_file` is misleadingly named because it also loads keystore accounts.
- Runtime signer polymorphism leaks into callers through `AccountVariant` and `with_account!`.
- Native private keys are stored unencrypted; only their display is hidden by default.
- Flattened untagged signer JSON preserves compatibility but gives weak errors and cannot robustly enforce mutually exclusive fields.
- Public-key matching is explicit for keystore and Ledger deployment, but not for ordinary deployed-account construction.
- Account contract type and signer type are orthogonal; all native account types can use local or Ledger signers.
- Native deployment uses `--name`, while normal signed commands use global `--account`.

## Technical implementation analysis

Technically, this is organized as a CLI-oriented architecture rather than a cohesive account domain. Command-specific code lives in vertical slices, while shared account types, persistence, runtime construction, and compatibility logic are spread across `lib.rs` and several helper modules.

### Package structure

```text
crates/sncast/src/
├── main.rs                         CLI definition, configuration, top-level dispatch
├── lib.rs                          Core types, account loading/building, RPC utilities
├── helpers/
│   ├── account.rs                  Accounts-file utilities and built-in devnet accounts
│   ├── signer.rs                   SignerType, SignerSource, AccountVariant
│   ├── configuration.rs            CastConfig and profile merging
│   ├── braavos.rs                  Custom Braavos account factory
│   └── ledger/
│       ├── account.rs              LedgerSigner and runtime account construction
│       ├── hd_path.rs              EIP-2645 parser
│       ├── key_locator.rs          CLI path/account-id resolution
│       ├── emulator_transport.rs   Speculos test transport
│       └── mod.rs                  Transport selection and exports
├── starknet_commands/
│   ├── account/
│   │   ├── mod.rs                  Account CLI orchestration and shared helpers
│   │   ├── create.rs
│   │   ├── import.rs
│   │   ├── deploy.rs
│   │   ├── list.rs
│   │   └── delete.rs
│   ├── ledger/                     Standalone `sncast ledger` commands
│   ├── declare.rs
│   ├── deploy.rs                   Contract deployment, not account deployment
│   ├── invoke.rs
│   └── multicall/
└── response/
    ├── account/                    Account command response DTOs
    └── ledger.rs
```

The package contains both a library and the `sncast` binary. The binary's private command modules import the package library as `sncast::...`. This creates two broad layers:

- `lib.rs` and `helpers`: reusable infrastructure and shared types.
- `main.rs` and `starknet_commands`: CLI commands and transaction workflows.

The division is incomplete: substantial account-domain behavior remains in both layers.

### Type organization

The main domain type, `AccountData`, lives directly in the large `lib.rs`, together with `AccountType`, network conversion, account loading, runtime construction, keystore parsing, execution encoding, and unrelated transaction helpers.

```rust
pub struct AccountData {
    pub public_key: Felt,
    pub address: Option<Felt>,
    pub salt: Option<Felt>,
    pub deployed: Option<bool>,
    pub class_hash: Option<Felt>,
    pub legacy: Option<bool>,
    pub account_type: Option<AccountType>,
    pub signer_type: SignerType,
}
```

It serves simultaneously as:

- the deserialization model for native accounts-file entries;
- the normalized output of starkli keystore parsing;
- the intermediate representation for devnet accounts;
- the input to runtime account construction;
- the input to account deployment.

Most fields are optional because different lifecycle stages and storage formats provide different subsets. The type does not encode states such as "undeployed but deployable" versus "deployed and usable"; each operation validates the fields it needs.

Signer-related types are isolated in `helpers/signer.rs`:

```text
SignerType
├── Local { private_key }
└── Ledger { ledger_path }

SignerSource
├── AccountsFile
├── Keystore(path)
└── Ledger(path)

AccountVariant
├── LocalWallet(SingleOwnerAccount<..., LocalWallet>)
└── Ledger(SingleOwnerAccount<..., LedgerSigner>)
```

These operate at different levels:

- `SignerType`: persisted signer configuration.
- `SignerSource`: input/backend routing.
- `AccountVariant`: constructed runtime account.

The distinction exists semantically but is obscured by their placement and naming.

### CLI and configuration entry point

Global CLI arguments are defined in `main.rs`. They are converted into `PartialCastConfig`, merged with local and global profiles, and normalized into `CastConfig`.

`CastConfig` carries unresolved account configuration:

```rust
pub account: String,
pub accounts_file: Utf8PathBuf,
pub keystore: Option<Utf8PathBuf>,
```

There is no typed account locator such as:

```text
Native { name, accounts_file }
Keystore { account_file, keystore }
Devnet { index }
```

Instead, downstream code interprets `account` according to the other fields. The same string can therefore mean an account name, a starkli file path, or a `devnet-N` selector.

Top-level transaction commands follow this pattern:

```text
CLI command
    ↓
resolve CastConfig and provider
    ↓
get_account(...)
    ↓
AccountVariant
    ↓
with_account!(...)
    ↓
generic command implementation
```

The entry points are in the large `run_async_command` match in `main.rs`.

### Runtime account construction

`get_account` is the principal resolver:

```text
configured account
    ├── devnet-N and no saved collision
    │       └── query devnet → AccountData(Local) → build local account
    └── normal/saved account
            ├── keystore configured
            │       └── parse starkli pair → AccountData(Local)
            └── no keystore
                    └── load network/name from accounts file
                            ├── private_key → local account
                            └── ledger_path → Ledger account
```

Native and keystore lookup converge at `get_account_from_accounts_file`, despite the function's name.

Local construction:

1. Extracts `private_key`.
2. Creates `SigningKey` and `LocalWallet`.
3. Requires `address`.
4. Checks that the address exists on the network.
5. Determines legacy/new execution encoding.
6. Constructs `SingleOwnerAccount`.
7. Sets its block to `pre_confirmed`.

Ledger construction performs the same address and encoding work, then delegates to `helpers/ledger/account.rs`. The local and Ledger builders duplicate some validation because their final generic types differ.

### Generic signer boundary

Transaction implementations are generic over the signer:

```rust
pub async fn invoke<S>(
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, S>,
    ...
)
where
    S: Signer + Sync + Send
```

This pattern is used by invoke, declare, contract deployment, and multicall. Transaction logic is written once and statically supports any compatible signer.

Rust cannot store the differently parameterized `SingleOwnerAccount` values as one concrete type. `AccountVariant` therefore holds both variants, and `with_account!` matches them at every call site:

```rust
with_account!(&account, |account| {
    invoke(..., account, ...).await
})
```

The inner command layer is signer-generic, but runtime polymorphism leaks into the top-level dispatcher through an enum and macro.

### Account lifecycle commands

Account management is grouped under `starknet_commands/account`. Its `mod.rs` owns several responsibilities:

- clap command definitions and dispatch;
- private-key CLI handling and validation;
- native account JSON construction;
- accounts-file writing;
- profile generation;
- interactive prompts;
- address computation for imports.

It is therefore both an orchestrator and a shared utility module.

#### Creation

`create.rs` selects the signer before selecting the account factory:

```text
SignerSource
    ├── Ledger → LedgerSigner + SignerType::Ledger
    └── otherwise → LocalWallet + SignerType::Local
                         ↓
              OZ / Ready / Braavos factory
                         ↓
          address + fee + account JSON
```

The `SignerSource::Keystore` branch still creates a local signer; its difference appears only when persisting the result.

Account factory selection is repeated in account creation, imported-address recomputation, and account deployment. This spreads the `AccountType → factory` mapping across multiple functions.

#### Import

`import.rs` directly produces `SignerType` and public key, then collects network metadata. It always writes the native accounts file. Keystore compatibility is not represented as an import implementation because starkli account and key files remain external inputs.

#### Account deployment

Account deployment in `account/deploy.rs` does not use `get_account` or `AccountVariant`. An undeployed account cannot be represented as a connected `SingleOwnerAccount` at an existing address. Deployment instead uses `AccountFactory<S>`:

```text
load undeployed AccountData
    ↓
match SignerType
    ├── LocalWallet
    └── LedgerSigner
    ↓
match AccountType
    ├── OpenZeppelinAccountFactory
    ├── ArgentAccountFactory
    └── BraavosAccountFactory
    ↓
deploy_v3
```

Keystore deployment is a parallel branch which loads the starkli pair, verifies public/private-key agreement, deploys, and rewrites the account JSON.

### Persistence organization

Persistence is the most scattered part of the implementation.

Native accounts-file operations are distributed across:

- generic JSON loading in `helpers/account.rs`;
- typed account lookup in `lib.rs`;
- account insertion in `starknet_commands/account/mod.rs`;
- deployment-status mutation in `account/deploy.rs`;
- deletion in `account/delete.rs`;
- listing and representation in `account/list.rs`.

There is no accounts repository or storage abstraction.

Writes are mostly performed through `serde_json::Value`, while reads deserialize into `AccountData`. This gives precise backward-compatible JSON output but sacrifices compile-time assurance that written records satisfy the read model.

Keystore compatibility is similarly divided:

- parsing and normalization: `lib.rs`;
- creation and serialization: `account/create.rs`;
- post-deployment mutation: `account/deploy.rs`;
- password/environment handling: `lib.rs` and constants.

The keystore account format is parsed manually with JSON pointers rather than a typed starkli schema. This makes partial compatibility easier but localizes format knowledge as string paths.

### Ledger organization

Ledger has a relatively clean internal split:

- `ledger/hd_path.rs`: path domain and validation.
- `ledger/key_locator.rs`: clap-facing locator resolution.
- `ledger/account.rs`: signer and connected-account construction.
- `ledger/mod.rs`: physical device versus emulator transport.
- `starknet_commands/ledger`: standalone public-key, version, and raw-signing commands.

The main coupling is `UI`: lower-level Ledger helpers print confirmation messages directly, so device infrastructure is not presentation-independent.

### Testing arrangement

Coverage follows the user-facing organization:

```text
crates/sncast/tests/
├── e2e/account/          create/import/deploy/list/delete
├── e2e/ledger/           basic device, account, and network flows
├── data/accounts/        native accounts-file fixtures
├── data/keystore/        starkli key/account fixtures
└── integration/          lower-level command behavior
```

Ledger end-to-end tests use Speculos through the `ledger-emulator` transport and are ignored unless the emulator is available. Unit tests for parsing and normalization also live inside `lib.rs` and the corresponding command modules.

### Overall assessment

The strongest technical decision is the signer-generic transaction layer: invoke, declare, deploy, and multicall do not duplicate their business logic for local and Ledger signers.

The weakest boundary is storage and resolution. Account selection, persistence backend, signer type, account lifecycle state, and runtime account construction are represented by partially overlapping types and spread across several modules.

The resulting architecture is:

```text
CLI-oriented vertical command slices
        +
central shared lib.rs utilities
        +
ad hoc persistence adapters
        +
generic starknet-rs transaction core
```

It is not currently organized as an explicit account domain with separate locator, storage, signer factory, account factory, and runtime account services.

## Physical file access and deserialization

There is no dedicated persistence/access layer. Account files and keystores are accessed directly with `std::fs` from several modules, using three different deserialization strategies:

- Native accounts file: strongly typed Serde model.
- Starkli account JSON: untyped `serde_json::Value` plus JSON pointers.
- Encrypted keystore: `starknet-rust`/`eth-keystore` typed Web3 keystore implementation.

### Path modelling and resolution

Paths enter through `CastConfig`:

```rust
pub account: String,
pub accounts_file: Utf8PathBuf,
pub keystore: Option<Utf8PathBuf>,
```

This is asymmetric:

- `accounts_file` is a typed path.
- `keystore` is a typed path.
- The starkli account-file path is stored in `account: String`, because that field normally contains a native account name.

All physical paths represented by `Utf8PathBuf` must be UTF-8.

Only the accounts-file path receives explicit tilde expansion:

```rust
let accounts_file =
    Utf8PathBuf::from(shellexpand::tilde(&accounts_file).to_string());
```

Neither `keystore` nor `account`, when it represents a starkli account path, receives equivalent expansion. Consequently:

- `accounts-file = "~/accounts.json"` works from configuration.
- `keystore = "~/key.json"` remains a literal path unless expanded externally.
- `account = "~/account.json"` also remains literal.
- Unquoted CLI `~` is normally expanded by the shell, while quoted values are not.

Relative paths are interpreted relative to the process working directory, not relative to the `snfoundry.toml` containing them.

Environment-variable values in configuration are expanded before deserialization, but only when the entire string begins with `$`, for example `$KEY_PATH` or `${KEY_PATH}`.

### Native accounts-file model

The native file is deserialized as:

```rust
HashMap<String, HashMap<String, AccountData>>
```

The physical structure is therefore enforced as:

```text
JSON object
└── network-name: JSON object
    └── account-name: AccountData object
```

The account model is:

```rust
pub struct AccountData {
    pub public_key: Felt,
    pub address: Option<Felt>,
    pub salt: Option<Felt>,
    pub deployed: Option<bool>,
    pub class_hash: Option<Felt>,
    pub legacy: Option<bool>,
    pub account_type: Option<AccountType>,

    #[serde(flatten)]
    pub signer_type: SignerType,
}
```

`public_key` and a recognizable signer are required. Other fields are optional during deserialization, although individual operations may later require them.

#### Felt representation

Because JSON is human-readable, every `Felt` field must be a string containing `0x`-prefixed hexadecimal:

```json
"public_key": "0x123"
```

Decimal JSON numbers and decimal strings are not accepted by the `Felt` Serde implementation. Serialization produces normalized hexadecimal strings.

#### Signer deserialization

`SignerType` is flattened and untagged:

```rust
#[serde(untagged)]
pub enum SignerType {
    Local { private_key: Felt },
    Ledger { ledger_path: DerivationPath },
}
```

Serde attempts the variants in declaration order:

1. An object containing a valid `private_key` becomes `Local`.
2. Otherwise, an object containing a valid `ledger_path` becomes `Ledger`.
3. Otherwise, deserialization fails with the generic “did not match any variant” error.

Consequences:

- There is no explicit signer discriminator.
- A hand-edited entry containing both fields is ambiguous; the first matching variant can win.
- Unknown fields are accepted because the models do not use `deny_unknown_fields`.
- Backward-compatible extra fields do not normally prevent loading.

Ledger paths are deserialized by the dependency's general BIP-32 `DerivationPath` parser. The stricter sncast EIP-2645 parser is used for CLI input but not when reading an existing accounts file. A hand-edited file can therefore contain a syntactically valid BIP-32 path which the CLI would reject.

### Native file reading

The main typed reader is `read_and_parse_json_file`:

```rust
let file_content = fs::read_to_string(path)?;

if file_content.trim().is_empty() {
    return Ok(T::default());
}

serde_path_to_error::deserialize(
    &mut serde_json::Deserializer::from_str(&file_content)
)
```

Properties:

- The complete file is read into memory as UTF-8.
- An empty native accounts file becomes an empty `HashMap`.
- JSON syntax and all account entries are validated in one pass.
- `serde_path_to_error` enriches errors with paths such as `alpha-sepolia.my-account.private_key`.
- Loading one account deserializes the entire file and clones the selected entry.

Because the whole file is typed, a malformed unrelated account under another network can prevent access to an otherwise valid account.

There is a second reader, `load_accounts`, which returns an untyped `serde_json::Value` and is used for mutation. Empty input becomes `{}` explicitly.

```text
Reads for execution    → typed NestedMap<AccountData>
Reads for modification → untyped serde_json::Value
```

### Native file writing and mutation

New entries are constructed manually as `serde_json::Value` in `prepare_account_json`, rather than by serializing `AccountData`. This gives exact control over compatibility field names, but makes the write model separate from the read model.

`write_account_to_accounts_file` performs insertion as follows:

1. If missing, create parent directories.
2. Create the file containing `{}`.
3. Read the complete file as `Value`.
4. Check the network/account slot.
5. Insert the new object.
6. Pretty-print and overwrite the complete file.

Deployment updates and deletion use the same read-modify-overwrite pattern.

Native file writes have no:

- advisory locking;
- atomic temporary-file-and-rename step;
- compare-and-swap/version check;
- `fsync`;
- backup;
- explicit file permissions.

Consequently:

- Concurrent sncast processes can overwrite each other's changes.
- A crash during truncation/write can leave a partial or empty JSON file.
- File permissions depend on the process umask.
- Native private keys may be created with broader permissions than desirable.
- Symlinks are followed by normal filesystem operations.

The existence check followed by creation is a time-of-check/time-of-use sequence rather than exclusive creation.

`account list` has an unusual error boundary: after checking that the file exists, rendering uses `read_and_flatten(...).unwrap_or_default()`. A malformed file can therefore be presented as containing no accounts instead of surfacing its parse error.

### Starkli account JSON model

The starkli account document is not represented by a Rust struct. It is loaded as `serde_json::Value` and interpreted manually in `get_account_data_from_keystore`.

Access uses JSON pointers:

```text
/deployment/address
/deployment/class_hash
/deployment/salt
/deployment/status
/variant/type
/variant/legacy
/variant/public_key
/variant/owner
/variant/signers/0/public_key
```

The result is normalized into native `AccountData` with:

```rust
signer_type: SignerType::Local { private_key }
```

Conceptually:

```text
starkli account Value + decrypted key
                  ↓
             AccountData
```

The generic `Value` reader validates only JSON syntax. Structural validation occurs incrementally afterward.

Most extraction follows:

```rust
json.pointer(pointer)
    .and_then(Value::as_str)
    .and_then(|value| value.parse().ok())
```

Malformed values are frequently converted into `None`, losing the original parsing error. A later context error may report a missing field even when it was present but malformed.

Additional behavior:

- Account JSON `version` is not validated.
- Unknown fields are tolerated.
- `deployment.status == "deployed"` becomes `true`.
- Any other status string becomes `false`.
- A missing status becomes `None`.
- `argent` is normalized to `ready`.
- Braavos requires `multisig.status == "off"` and exactly one signer.

This is looser and more compatibility-oriented than native `AccountData` deserialization.

### Encrypted keystore model

The encrypted key file is delegated to `starknet-rust`:

```rust
SigningKey::from_keystore(path, password)
```

The implementation uses `eth-keystore`'s typed `EthKeystore` model:

```rust
struct EthKeystore {
    crypto: CryptoJson,
    id: Uuid,
    version: u8,
}
```

`CryptoJson` contains:

- cipher name;
- IV;
- ciphertext;
- KDF type and parameters;
- MAC.

Hex-encoded byte fields are decoded by custom Serde functions.

Decryption supports:

- scrypt;
- PBKDF2;
- AES-128-CTR encryption;
- Keccak-based MAC verification.

A wrong password normally fails through MAC mismatch before decrypted bytes are accepted.

The password comes from:

- `KEYSTORE_PASSWORD` when loading;
- `CREATE_KEYSTORE_PASSWORD` when creating;
- otherwise a hidden terminal prompt.

### Keystore creation

`create_to_keystore` first checks that neither target exists, then:

1. Writes the encrypted keystore through `SigningKey::save_as_keystore`.
2. Constructs the starkli account document as `serde_json::Value`.
3. Creates the account document's parent directories.
4. Pretty-prints the account JSON.

The generated keystore is Web3 Secret Storage version 3, using:

- scrypt with `N = 8192`, `r = 8`, `p = 1`;
- AES-128-CTR;
- random salt and IV;
- compact JSON output.

An implementation detail in `starknet-rust` splits the requested keystore path into parent directory and filename before calling `eth-keystore`. It does not create the parent directory. The keystore parent directory must therefore already exist, even though the starkli account-file parent is created automatically.

Creation is not transactional. The keystore is written before the account JSON. If the second write fails, an orphaned keystore remains.

File creation relies on normal `File::create`, with permissions controlled by umask rather than explicit secret-file permissions.

### Starkli account mutation

After successful deployment, `update_keystore_account`:

1. Reads the complete account document.
2. Deserializes the root as `Map<String, Value>`.
3. Sets deployment status to `deployed`.
4. Removes `salt` and `context`.
5. Adds the deployed address.
6. Pretty-prints and overwrites the complete file.

Unknown top-level and nested fields are mostly preserved, except for the explicitly removed deployment fields. Like native mutation, this operation is unlocked and non-atomic.

### Simple private-key files

`--private-key-file` is a fourth, much simpler filesystem format. It is not JSON:

```rust
let private_key_string = std::fs::read_to_string(path)?;
let key: Felt = private_key_string.parse()?;
```

It expects the complete contents to parse directly as a felt. There is no explicit trimming before parsing, so trailing-whitespace behavior depends on `Felt::from_str`. The key is subsequently checked to be nonzero and smaller than the Stark curve order.

### Physical-access assessment

Physical access is implemented as direct whole-file operations:

```text
Path from config/CLI
    ↓
read complete UTF-8 file
    ↓
typed Serde / untyped Value / eth-keystore
    ↓
normalize into AccountData
```

The modelling quality differs by format:

| Format | Model strength | Error quality |
| --- | --- | --- |
| Native accounts file | Strongly typed nested maps and `AccountData` | Good field paths, but untagged signer errors are vague |
| Starkli account JSON | Untyped `Value` plus JSON pointers | Compatibility-friendly but often loses malformed-value details |
| Encrypted keystore | Typed Web3 keystore schema in dependency | Strong crypto/schema errors |

The main physical-access weaknesses are the lack of an abstraction boundary, whole-file unlocked mutation, non-atomic writes, implicit permissions, inconsistent path expansion, and separate read/write models.
