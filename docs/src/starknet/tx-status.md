# Inspecting Transactions

## Overview

Starknet Foundry `sncast` supports inspecting transaction statuses with `sncast get tx-status` and transaction execution traces with `sncast get tx-trace`.

For detailed CLI descriptions, refer to the [get tx-status](../appendix/sncast/get/tx-status.md) and [get tx-trace](../appendix/sncast/get/tx-trace.md) command references.

## Usage Examples

### Inspecting Transaction Status

You can track the details about the execution and finality status of a transaction in the given network by using the transaction hash as shown below:

```shell
$ sncast \
 get tx-status \
 0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1 \
 --network sepolia
```

<details>
<summary>Output:</summary>

```shell
Success: Transaction status retrieved

Finality Status:  Accepted on L1
Execution Status: Succeeded
```

</details>

### Inspecting a Transaction Trace

Use `get tx-trace` to display validation, execution or constructor, L1 handler, and fee transfer calls as aligned, nested fields. For Cairo 1 contracts, `sncast` uses the contract ABI to decode selectors, calldata, and call results.

```shell
$ sncast \
 get tx-trace \
 0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1 \
 --network sepolia
```

<details>
<summary>Output:</summary>

```shell
Success: Transaction trace retrieved

Type:                     INVOKE
Transaction Hash:         0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1
Validate Invocation
  Entry Point Selector:   __validate__
  Contract Address:       0x[..]
  Calldata:               array![..]
  Result:                 success: 0x56414c4944
Execute Invocation
  Entry Point Selector:   __execute__
  Contract Address:       0x[..]
  Calldata:               array![..]
  Result:                 success: array![array![].span()]
  Calls
    Entry Point Selector: transmit
    Contract Address:     0x[..]
    Calldata:             ReportContext { config_digest: 0x[..] }, [..]
    Result:               success
Fee Transfer Invocation
  Entry Point Selector:   transfer
  Contract Address:       0x[..]
  Calldata:               ContractAddress(0x[..]), [..]_u256
  Result:                 success: true
```

</details>

If a class or ABI cannot be fetched, or a value cannot be decoded, the affected selector and felts are displayed in hexadecimal instead. Cairo 0 selector names are resolved when an ABI is available, while their calldata and results remain raw.

Pass the global `--json` flag to return the complete, unmodified `starknet_traceTransaction` result under the `transaction_trace` field. JSON mode does not perform ABI lookups.

Pass `--full` to include every trace field using the same aligned field format as `get tx-receipt`, while preserving the nesting of the Starknet Trace API schema.
