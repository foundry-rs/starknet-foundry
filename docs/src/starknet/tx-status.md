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

By default, the command displays a compact trace focused on the invocation flow. For each invocation, it shows the entry point selector, contract address, calldata, result, and any nested calls.

Use the `--full` flag to include the remaining trace data, such as call and entry point types, caller and class information, emitted events, L1 messages, execution resources, revert status, and the transaction state diff.

```shell
$ sncast \
 get tx-trace \
 0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1 \
 --full \
 --network sepolia
```

<details>
<summary>Output:</summary>

```shell
Success: Transaction trace retrieved

Type:                     INVOKE
Execute Invocation
  Call Type:              CALL
  Calldata:               array![Call[..]
  Caller Address:         0x0
  Calls
    Call Type:            CALL
    Calldata:             ReportContext[..]
    Caller Address:       0x[..]
    Calls:                []
    Class Hash:           0x[..]
    Contract Address:     0x[..]
    Entry Point Selector: transmit
    Entry Point Type:     EXTERNAL
    Events
      Data:               [[..]]
      Keys:               [[..]]
      Order:              0
    Execution Resources
      L1 Gas:             18
      L2 Gas:             0
    Is Reverted:          false
    Messages:             []
    Result:               success
  Class Hash:             0x[..]
  Contract Address:       0x[..]
  Entry Point Selector:   __execute__
  Entry Point Type:       EXTERNAL
  Events:                 []
  Execution Resources
    L1 Gas:               18
    L2 Gas:               0
  Is Reverted:            false
  Messages:               []
  Result:                 success: array![array![].span()]
Execution Resources
  L1 Data Gas:            640
  L1 Gas:                 32
  L2 Gas:                 0
Fee Transfer Invocation
  Call Type:              CALL
  Calldata:               ContractAddress(0x[..]), 4014902418114130240_u256
  Caller Address:         0x[..]
  Calls:                  []
  Class Hash:             0x[..]
  Contract Address:       0x[..]
  Entry Point Selector:   transfer
  Entry Point Type:       EXTERNAL
  Events
    Data:                 [[..]]
    Keys:                 [[..]]
    Order:                0
  Execution Resources
    L1 Gas:               4
    L2 Gas:               0
  Is Reverted:            false
  Messages:               []
  Result:                 success: true
Validate Invocation
  Call Type:              CALL
  Calldata:               array![Call[..]
  Caller Address:         0x0
  Calls:                  []
  Class Hash:             0x[..]
  Contract Address:       0x[..]
  Entry Point Selector:   __validate__
  Entry Point Type:       EXTERNAL
  Events:                 []
  Execution Resources
    L1 Gas:               8
    L2 Gas:               0
  Is Reverted:            false
  Messages:               []
  Result:                 success: 0x56414c4944
```

</details>
