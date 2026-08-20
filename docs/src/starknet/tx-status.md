# Inspecting Transactions

## Overview

Starknet Foundry `sncast` supports the inspection of transaction statuses on a given network with the `sncast get tx-status` command.

For a detailed CLI description, refer to the [get tx-status command reference](../appendix/sncast/get/tx-status.md).

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

Use `get tx-trace` to display the execution trace of a transaction.

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

By default, the trace output omits some fields for brevity. To display the full transaction trace, use the `--full` flag:

```shell
$ sncast \
 get tx-trace \
 0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1 \
 --full \
 --network sepolia
```
