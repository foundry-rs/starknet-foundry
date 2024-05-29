# Inspecting Transactions

## Overview

Starknet Foundry `sncast` supports the inspection of transaction statuses on a given network with the `sncast tx-status` command.

For a detailed CLI description, refer to the [tx-status command reference](../appendix/sncast/tx-status.md).

## Usage Examples

### Inspecting Transaction Status

You can track the details about the execution and finality status of a transaction in the given network by using the transaction hash as shown below:

```shell
$ sncast \
 --url http://127.0.0.1:5050  \
 tx-status \
 0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1 

command: tx-status
execution_status: SUCCEEDED
finality_status: ACCEPTED_ON_L2
```

