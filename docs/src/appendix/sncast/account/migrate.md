# `migrate`

Upgrade the selected accounts file from the legacy unversioned schema to schema version 2.

The migration preserves account metadata, converts implicit private-key and Ledger fields to tagged signer objects, and writes a same-directory `.v1.bak` backup before replacing the original. Reading a V1 file never migrates it. A successful create, import, deploy, or delete mutation may also perform the same upgrade automatically.
