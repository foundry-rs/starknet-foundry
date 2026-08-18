# sncast account and signer architecture refactor

## Objective

Refactor sncast so that the accounts file is explicitly versioned, signer definitions use a stable tagged schema, keystores are native signers, and filesystem persistence, schema decoding, domain logic, signer construction, and command execution have clear boundaries.

The intended end state is:

- The accounts file is a versioned persistence format.
- Accounts are canonical domain objects independent of any schema version.
- Signers are independently resolvable capabilities.
- Commands consume a validated account and a runtime signer.
- Commands do not know about JSON, schema versions, filesystem behavior, or starkli compatibility rules.

## Target architecture

Organize the code around the two domain concepts and their boundaries:

```text
crates/sncast/src/
├── accounts/
│   ├── mod.rs
│   ├── domain.rs            AccountRegistry, AccountRecord, validated views
│   ├── selector.rs          Native, devnet, legacy-starkli selection
│   ├── repository.rs        Account lookup and mutation API
│   ├── service.rs           Runtime account construction
│   ├── deployment.rs        Account factory/address/deployment logic
│   ├── schema/
│   │   ├── mod.rs           VersionedAccountsFile and version dispatch
│   │   ├── v1.rs            Exact legacy schema
│   │   ├── v2.rs            New explicit schema
│   │   └── migration.rs     V1 to canonical domain/V2
│   └── storage/
│       ├── mod.rs
│       └── filesystem.rs
├── signers/
│   ├── mod.rs
│   ├── spec.rs
│   ├── backend.rs
│   ├── runtime.rs
│   ├── resolver.rs
│   ├── credentials.rs
│   ├── private_key.rs
│   ├── keystore.rs
│   └── ledger.rs
└── compat/
    └── starkli.rs
```

The normal execution path should be:

```text
CLI parsing
  -> AccountSelector
  -> AccountService / AccountRepository
  -> canonical AccountRecord
  -> SignerResolver
  -> RuntimeSigner
  -> SingleOwnerAccount<RuntimeSigner>
  -> command
```

## 1. Establish a canonical account domain

Move account-domain types such as `AccountType` and `AccountData` out of the broad `lib.rs` module. Replace persistence-shaped values with an explicit canonical model:

```rust
struct AccountRegistry {
    networks: BTreeMap<NetworkName, BTreeMap<AccountName, AccountRecord>>,
}

struct AccountRecord {
    public_key: Felt,
    address: Option<Felt>,
    salt: Option<Felt>,
    deployed: Option<bool>,
    class_hash: Option<Felt>,
    execution_encoding: Option<ExecutionEncoding>,
    account_type: Option<AccountType>,
    signer: SignerSpec,
}
```

Use `BTreeMap` so serialized output and diagnostics are deterministic. Introduce newtypes such as `NetworkName` and `AccountName` where they prevent accidental mixing of identifiers.

An account record can represent different lifecycle stages, so do not force every field to exist globally. Instead, validate it for the requested operation and return capability-specific views:

```rust
impl AccountRecord {
    fn as_connected(&self) -> Result<ConnectedAccountRecord, AccountsError>;
    fn as_deployable(&self) -> Result<DeployableAccountRecord, AccountsError>;
}
```

This keeps persistence flexible while ensuring that transaction code receives complete, validated data.

## 2. Define an explicit V2 accounts schema

Every V2 account must contain a tagged `signer` object. A representative file is:

```json
{
  "version": 2,
  "accounts": {
    "alpha-sepolia": {
      "alice": {
        "public_key": "0x123",
        "address": "0x456",
        "salt": "0x789",
        "deployed": true,
        "class_hash": "0xabc",
        "legacy": false,
        "type": "open_zeppelin",
        "signer": {
          "type": "keystore",
          "path": "keys/alice.json",
          "password_env": "ALICE_KEYSTORE_PASSWORD"
        }
      }
    }
  }
}
```

The schema DTO belongs in `accounts/schema/v2.rs` and should use strict deserialization, including `deny_unknown_fields` where appropriate. The schema-level signer is externally tagged by a readable discriminator:

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Signer {
    PrivateKey {
        private_key: Felt,
    },
    Keystore {
        path: Utf8PathBuf,
        password_env: Option<String>,
    },
    Ledger {
        derivation_path: DerivationPath,
    },
}
```

Use `private_key` rather than `local`, because it describes the credential rather than its implementation. Give Ledger derivation paths a strict custom serializer and deserializer for the supported EIP-2645 representation instead of treating them as arbitrary strings.

## 3. Make schema versioning an explicit boundary

Preserve the current accounts-file representation exactly in `accounts/schema/v1.rs`, including its flattened, untagged signer variants. Untagged signer deserialization must not appear anywhere outside the V1 compatibility schema.

Expose a versioned persistence enum:

```rust
enum VersionedAccountsFile {
    V1(v1::AccountsFile),
    V2(v2::AccountsFile),
}
```

Implement version dispatch with these rules:

- A document without a `version` field is V1.
- A document with `"version": 2` is V2.
- Any other version produces a dedicated unsupported-version error.
- Both variants normalize immediately into `AccountRegistry`.
- No downstream code receives `VersionedAccountsFile` or version-specific DTOs.
- All new serialization uses V2 only.

Keep the dispatch implementation local to the schema module. It may inspect a small JSON envelope to choose a typed decoder, but the rest of the application must not manipulate `serde_json::Value`.

## 4. Provide deliberate V1-to-V2 migration

Use controlled upgrade-on-write semantics:

- Reading a V1 file must not modify it.
- A successful mutation of a V1 registry writes the resulting document as V2.
- Before the first conversion, create a same-directory V1 backup with safe permissions.
- Notify the user when an automatic conversion occurs.
- Provide an explicit `sncast account migrate` command for users who want to upgrade without another mutation.
- Never rewrite a file merely because it was read, or after a failed operation.

The migration mapping is deterministic:

- A V1 `private_key` signer becomes `{"type":"private_key","private_key":...}`.
- A V1 `ledger_path` signer becomes `{"type":"ledger","derivation_path":...}`.
- Existing account metadata is preserved without semantic changes.
- Ledger paths are validated under the V2 rules during conversion, with an error identifying the account that cannot be migrated.

Migration should be implemented as pure conversion functions from V1 DTOs to the canonical domain, and from the canonical domain to V2 DTOs. This makes it independently testable and avoids coupling it to filesystem writes.

## 5. Separate codec, storage, and repository responsibilities

Introduce three distinct layers.

### Accounts codec

`AccountsCodec` is pure and receives or returns bytes or strings:

```rust
trait AccountsCodec {
    fn decode(&self, input: &[u8]) -> Result<DecodedRegistry, AccountsError>;
    fn encode_v2(&self, registry: &AccountRegistry) -> Result<Vec<u8>, AccountsError>;
}
```

`DecodedRegistry` carries the canonical registry plus source-version metadata needed to decide whether a future successful write is a migration. It performs no I/O.

### Accounts storage

`AccountsStorage` owns physical persistence:

```rust
trait AccountsStorage {
    fn read(&self, path: &Utf8Path) -> Result<Vec<u8>, AccountsError>;
    fn write_atomic(&self, path: &Utf8Path, bytes: &[u8]) -> Result<(), AccountsError>;
    fn with_exclusive_lock<T>(
        &self,
        path: &Utf8Path,
        operation: impl FnOnce() -> Result<T, AccountsError>,
    ) -> Result<T, AccountsError>;
}
```

The production filesystem implementation is responsible for:

- Creating parent directories deliberately.
- Holding a sidecar exclusive lock for the complete read-modify-write transaction.
- Writing a temporary file in the destination directory.
- Flushing and syncing file contents before replacement.
- Atomically renaming the temporary file over the destination.
- Syncing the parent directory where supported.
- Applying owner-only permissions such as `0600` on Unix.
- Defining and testing whether symlinks are rejected or resolved.
- Avoiding the current pattern of creating a preliminary `{}` file.

### Account repository

`AccountRepository` combines storage and codec and exposes domain operations such as `load`, `find`, and `mutate`. Its mutation transaction owns the sequence:

```text
acquire lock
  -> read
  -> decode/version-dispatch
  -> normalize
  -> apply domain mutation
  -> encode V2
  -> atomic write
  -> release lock
```

No command module should call `std::fs`, deserialize an accounts file, call a global `load_accounts`, or mutate `serde_json::Value` directly.

## 6. Promote signer configuration into the domain

Define a schema-independent signer specification:

```rust
enum SignerSpec {
    PrivateKey(PrivateKeySpec),
    Keystore(KeystoreSpec),
    Ledger(LedgerSpec),
}
```

The domain types should not derive Serde. Schema DTOs convert to and from them, preserving the option to evolve the file format independently of runtime interfaces.

`KeystoreSpec` contains the keystore path and an optional password-environment variable name. Relative paths resolve relative to the directory containing the accounts file; absolute paths remain absolute. This rule must be applied in one resolver and documented, rather than depending on the process working directory.

Resolve a keystore password in this order:

1. The signer's `password_env` variable.
2. `SNCAST_KEYSTORE_PASSWORD`.
3. The legacy `KEYSTORE_PASSWORD` during the compatibility period.
4. An interactive hidden prompt when standard input is interactive.
5. A clear credential-unavailable error in non-interactive execution.

Never persist a password in the accounts file or another sncast configuration file.

## 7. Add an extensible runtime signer abstraction

The starknet-rust `Signer` trait has an associated error type, which makes heterogeneous signer dispatch awkward. Contain that constraint in one internal object-safe interface:

```rust
#[async_trait]
trait SignerBackend: Send + Sync {
    async fn public_key(&self) -> Result<VerifyingKey, SignerError>;
    async fn sign_hash(&self, hash: &Felt) -> Result<Signature, SignerError>;
    fn is_interactive(&self) -> bool;
    fn kind(&self) -> SignerKind;
}
```

Then provide one concrete adapter:

```rust
struct RuntimeSigner {
    backend: Arc<dyn SignerBackend>,
}
```

`RuntimeSigner` implements the starknet-rust `Signer` trait once and maps every backend failure into a unified `SignerError`.

Initial backends are:

- `PrivateKeyBackend`, using a plain locally available signing key.
- `KeystoreBackend`, retaining keystore identity for diagnostics and policy while delegating cryptographic signing to the decrypted local wallet.
- `LedgerBackend`, wrapping the Ledger signer and its interaction semantics.

`SignerResolver` converts a `SignerSpec` into a `RuntimeSigner`. It owns credential retrieval, user interaction, and backend/provider registration. Adding a future signer should require only:

- A new schema and domain spec variant.
- A backend implementation.
- A resolver/provider registration.
- Focused schema, resolver, and backend tests.

It must not require changes in transaction submission or account command logic.

## 8. Use one runtime account type

After signer erasure, use one account type:

```rust
type RuntimeAccount<'a> =
    SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, RuntimeSigner>;
```

Remove `AccountVariant`, `with_account!`, duplicated local-versus-Ledger dispatch, and generic signer plumbing that exists only to accommodate concrete signer types.

`AccountService` coordinates the repository, signer resolver, starkli compatibility adapter, and devnet source. Its primary operations should be capability-oriented, for example:

```rust
async fn connected_account(
    &self,
    selector: &AccountSelector,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<RuntimeAccount<'_>, AccountsError>;

async fn deployable_account(
    &self,
    selector: &AccountSelector,
) -> Result<ResolvedDeployableAccount, AccountsError>;
```

Centralize in this service:

- Selector and network resolution.
- Account lookup and lifecycle validation.
- Signer resolution.
- Signer public-key verification.
- Address and execution-encoding checks.

Always verify that a private-key or keystore signer derives the public key recorded by the account. Centralize Ledger verification as well, avoiding unnecessary device-display prompts during ordinary use while retaining explicit confirmation during creation or import.

## 9. Centralize account deployment behavior

Create an `AccountDeploymentService` in `accounts/deployment.rs` with operations such as:

- `compute_address`
- `estimate_fee`
- `deploy`

This should be the only module that matches `AccountType` to the appropriate OpenZeppelin, Argent/Ready, or Braavos account factory. Account creation, import validation, and deployment then share the same address and factory logic rather than maintaining parallel matches.

## 10. Make keystore-native workflows ordinary account workflows

Support native creation directly into the V2 account registry, for example:

```text
sncast account create --name alice --keystore keys/alice.json
```

The workflow should:

1. Generate or accept a private key.
2. Encrypt and write the keystore.
3. Store a tagged `keystore` signer in the native account entry.
4. Avoid creating a starkli account JSON file.

Importing an existing encrypted key should follow the same model. Subsequent commands use only `--account alice`; they do not require a global keystore option.

If users still need starkli-format account output, expose that as an explicitly named compatibility export operation rather than embedding it in the native model.

## 11. Isolate legacy starkli compatibility

Move starkli account-file parsing out of `lib.rs` and into `compat/starkli.rs`. The adapter must produce the same canonical `AccountRecord` and `SignerSpec` used by native accounts.

Temporarily retain the global combination `--keystore key.json --account account.json` as a deprecated selector:

```rust
enum AccountSelector {
    Named {
        name: AccountName,
        accounts_file: Utf8PathBuf,
    },
    Devnet {
        index: NonZeroU8,
    },
    LegacyStarkli {
        account_file: Utf8PathBuf,
        keystore_file: Utf8PathBuf,
    },
}
```

Construct this selector at the CLI/configuration boundary. Downstream code must not infer account kinds from string prefixes or decide whether an `--account` value is a name or path.

Add `sncast account import-starkli` to convert the legacy pair into a native V2 entry with a keystore signer. This supplies an intentional migration path before removal of:

- The global `--keystore` field.
- The overloaded account-name/account-path behavior.
- Keystore branches in the current `get_account` path.
- Direct mutation of starkli account JSON.

New profiles for a keystore-backed native account should contain only the account name and accounts-file location, like every other native signer.

## 12. Introduce typed errors at each boundary

Use structured errors within the account and signer subsystems and convert them to user-facing `anyhow` context only at the command boundary.

`AccountsError` should distinguish at least:

- Storage failures.
- Invalid schema.
- Unsupported schema version.
- Migration failure.
- Account not found or duplicate account.
- Invalid account state for an operation.
- Signer resolution failure.

`SignerError` should distinguish at least:

- Unsupported signer kind.
- Credential unavailable.
- Invalid keystore password or keystore data.
- Public-key mismatch.
- Device unavailable.
- User rejection.
- Backend signing failure.

Errors should carry relevant context such as file path, schema version, network, account name, signer kind, and invalid field. Secret material and password values must never appear in diagnostics.

## 13. Testing strategy

Build tests around boundaries rather than only command behavior.

### Schema and migration tests

- Golden fixtures for every existing V1 private-key and Ledger shape.
- V2 round trips for every signer variant.
- Rejection of ambiguous, unknown, or mixed signer fields.
- Rejection of unknown versions and invalid Ledger paths.
- Deterministic V1-to-domain-to-V2 snapshots.
- Confirmation that read-only V1 operations do not rewrite files.

### Storage and repository tests

- Missing file and missing parent behavior.
- Locking of concurrent mutations.
- Atomic replacement and preservation after simulated failures.
- Backup creation on first V1 mutation.
- File-permission behavior on Unix.
- Defined symlink behavior.
- No mutation when validation or command preparation fails.

### Signer tests

- Uniform transaction-facing behavior for private-key, keystore, and Ledger backends.
- Password-source precedence and non-interactive failure.
- Relative keystore path resolution.
- Public-key mismatch detection.
- Interactive signer policy.
- Backend error mapping into stable typed errors.

### Service and command tests

- Named, devnet, and legacy starkli selectors enter the same canonical pipeline.
- Commands do not branch on signer variants.
- Create, import, deploy, show, delete, and transaction commands use repository/service APIs only.
- Existing user-visible behavior remains characterized until a deliberate migration or deprecation changes it.

## 14. Ordered implementation plan

1. Characterize and freeze current V1 behavior with golden fixtures and integration tests before moving code.
2. Introduce the canonical account domain, identifier newtypes, signer specifications, and validated operation views.
3. Build the codec, filesystem storage, and repository layers; move all raw accounts-file access behind them.
4. Implement exact V1 and strict V2 schemas, custom version dispatch, pure normalization, and V1-to-V2 migration.
5. Harden persistence with whole-transaction locking, atomic replacement, safe permissions, backups, symlink policy, and contextual errors.
6. Introduce `SignerBackend`, `RuntimeSigner`, and `SignerResolver` for private-key and Ledger signers; remove `AccountVariant`, `with_account!`, and duplicated dispatch.
7. Add the first-class keystore signer, native create/import flows, deterministic path resolution, credential providers, and public-key verification.
8. Move starkli behavior into the compatibility adapter, deprecate legacy CLI/config fields, and add `account import-starkli`.
9. Centralize account-type factory matching and address/deployment logic in `AccountDeploymentService`.
10. Finish migrating every command so it depends only on selectors, repositories, services, validated domain views, and `RuntimeAccount`; update profiles, `show-config`, documentation, examples, and shell completions.
11. Delete transitional types and functions, remove obsolete raw JSON/filesystem helpers, and reduce `lib.rs` to intentional public wiring.

Each step should leave the repository compiling and keep compatibility tests passing. Transitional adapters may exist briefly, but new command paths must target the end-state interfaces rather than adding behavior to the old abstractions.

## Completion criteria

The refactor is complete when:

- Every newly written native accounts file declares version 2.
- Every V2 account has exactly one explicitly tagged signer.
- Existing V1 files remain readable and can be migrated safely and intentionally.
- Command modules do not access account files or schema DTOs directly.
- Native accounts transparently use private-key, keystore, or Ledger signers.
- Adding another signer does not change transaction command code.
- `AccountVariant` and `with_account!` no longer exist.
- Starkli compatibility is isolated behind a named adapter and migration workflow.
- Account-file mutations are locked, atomic, permission-aware, and protected during migration.
- The schema, domain, runtime signer, persistence, compatibility, and CLI layers are discoverable from the module structure and have focused tests.
