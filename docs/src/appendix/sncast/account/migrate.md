# `migrate`

Upgrade the selected accounts file from the legacy unversioned schema to schema version 2.

The migration preserves account metadata, converts implicit private-key and Ledger fields to tagged signer objects, and writes a same-directory `.v1.bak` backup before replacing the original. Reading a V1 file never migrates it. A successful create, import, deploy, or delete mutation may also perform the same upgrade automatically.

An accounts file already using schema V2 is left byte-for-byte unchanged, without creating a backup. A zero-byte or whitespace-only file contains no account data to back up: migration initializes an empty V2 accounts file without a backup. The accounts file must exist before running this command.

Writes use a same-directory sidecar lockfile to coordinate concurrent writers. The lock is released when the command finishes, but the lockfile may remain on disk. Do not remove it while writers may be using the accounts file, since they must lock the same file.
