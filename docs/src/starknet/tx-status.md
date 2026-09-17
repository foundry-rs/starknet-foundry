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
  Contract Address:       0x01d091b30a2d20ca2509579f8beae26934bfdc3725c0b497f50b353b7a3c636f
  Calldata:               array![Call { to: ContractAddress(0x424ce41bea300e095e763d9fb4316af76c9da9c0fa926009f25b42b6f4ad04a), selector: 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8, calldata: array![0x441b0ab7fcd3923bd830e146e99ed90c4aebd19951eb6ed7b3713241aa8af, 0x29e701, 0xf29c0193adc354752489f1a7af2f507d72a5e5b76cce705094d05d72e21ab5, 0x6655cb7c, 0x304020100000000000000000000000000000000000000000000000000000000, 0x4, 0x27693e402, 0x276a2b3d2, 0x276a3f070, 0x276aeecf0, 0xb9eab07caffbd5538, 0x1, 0x2, 0x6771e459d1e5563ec13af0ca40f04406ff4b70e6cc9a534dce12957f46c0f24, 0x36383aebe2151145a66dd7a87d9c885a862339e35d2ee0bd9df4075d17a8979, 0x2cb74dff29a13dd5d855159349ec92f943bacf0547ff3734e7d84a15d08cbc5, 0xb1a29e2cfed2f0a9d5f137845280bb6ce746f2f4b6a2dd05ec794171f4012, 0x1f85c957582717816bd2c910ac678caf007f6f84d71bc5a95f38de0b6435163, 0x4225d1c8ee8e451a25e30c10689ef898e11ccf5c0f68d0fc7876c47b318e946].span() }]
  Result:                 success: 0x56414c4944
Execute Invocation
  Entry Point Selector:   __execute__
  Contract Address:       0x01d091b30a2d20ca2509579f8beae26934bfdc3725c0b497f50b353b7a3c636f
  Calldata:               array![Call { to: ContractAddress(0x424ce41bea300e095e763d9fb4316af76c9da9c0fa926009f25b42b6f4ad04a), selector: 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8, calldata: array![0x441b0ab7fcd3923bd830e146e99ed90c4aebd19951eb6ed7b3713241aa8af, 0x29e701, 0xf29c0193adc354752489f1a7af2f507d72a5e5b76cce705094d05d72e21ab5, 0x6655cb7c, 0x304020100000000000000000000000000000000000000000000000000000000, 0x4, 0x27693e402, 0x276a2b3d2, 0x276a3f070, 0x276aeecf0, 0xb9eab07caffbd5538, 0x1, 0x2, 0x6771e459d1e5563ec13af0ca40f04406ff4b70e6cc9a534dce12957f46c0f24, 0x36383aebe2151145a66dd7a87d9c885a862339e35d2ee0bd9df4075d17a8979, 0x2cb74dff29a13dd5d855159349ec92f943bacf0547ff3734e7d84a15d08cbc5, 0xb1a29e2cfed2f0a9d5f137845280bb6ce746f2f4b6a2dd05ec794171f4012, 0x1f85c957582717816bd2c910ac678caf007f6f84d71bc5a95f38de0b6435163, 0x4225d1c8ee8e451a25e30c10689ef898e11ccf5c0f68d0fc7876c47b318e946].span() }]
  Result:                 success: array![array![].span()]
  Calls
    Entry Point Selector: transmit
    Contract Address:     0x0424ce41bea300e095e763d9fb4316af76c9da9c0fa926009f25b42b6f4ad04a
    Calldata:             ReportContext { config_digest: 0x441b0ab7fcd3923bd830e146e99ed90c4aebd19951eb6ed7b3713241aa8af, epoch_and_round: 2746113_u64, extra_hash: 0xf29c0193adc354752489f1a7af2f507d72a5e5b76cce705094d05d72e21ab5 }, 1716898684_u64, 0x304020100000000000000000000000000000000000000000000000000000000, array![10579338242_u128, 10580308946_u128, 10580390000_u128, 10581110000_u128], 214347425458270590264_u128, 1_u128, array![Signature { r: 0x6771e459d1e5563ec13af0ca40f04406ff4b70e6cc9a534dce12957f46c0f24, s: 0x36383aebe2151145a66dd7a87d9c885a862339e35d2ee0bd9df4075d17a8979, public_key: 0x2cb74dff29a13dd5d855159349ec92f943bacf0547ff3734e7d84a15d08cbc5 }, Signature { r: 0xb1a29e2cfed2f0a9d5f137845280bb6ce746f2f4b6a2dd05ec794171f4012, s: 0x1f85c957582717816bd2c910ac678caf007f6f84d71bc5a95f38de0b6435163, public_key: 0x4225d1c8ee8e451a25e30c10689ef898e11ccf5c0f68d0fc7876c47b318e946 }]
    Result:               success
Fee Transfer Invocation
  Entry Point Selector:   transfer
  Contract Address:       0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
  Calldata:               ContractAddress(0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8), 4014902418114130240_u256
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
  Calldata:               array![Call { to: ContractAddress(0x424ce41bea300e095e763d9fb4316af76c9da9c0fa926009f25b42b6f4ad04a), selector: 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8, calldata: array![0x441b0ab7fcd3923bd830e146e99ed90c4aebd19951eb6ed7b3713241aa8af, 0x29e701, 0xf29c0193adc354752489f1a7af2f507d72a5e5b76cce705094d05d72e21ab5, 0x6655cb7c, 0x304020100000000000000000000000000000000000000000000000000000000, 0x4, 0x27693e402, 0x276a2b3d2, 0x276a3f070, 0x276aeecf0, 0xb9eab07caffbd5538, 0x1, 0x2, 0x6771e459d1e5563ec13af0ca40f04406ff4b70e6cc9a534dce12957f46c0f24, 0x36383aebe2151145a66dd7a87d9c885a862339e35d2ee0bd9df4075d17a8979, 0x2cb74dff29a13dd5d855159349ec92f943bacf0547ff3734e7d84a15d08cbc5, 0xb1a29e2cfed2f0a9d5f137845280bb6ce746f2f4b6a2dd05ec794171f4012, 0x1f85c957582717816bd2c910ac678caf007f6f84d71bc5a95f38de0b6435163, 0x4225d1c8ee8e451a25e30c10689ef898e11ccf5c0f68d0fc7876c47b318e946].span() }]
  Caller Address:         0x0000000000000000000000000000000000000000000000000000000000000000
  Class Hash:             0x066559c86e66214ba1bc5d6512f6411aa066493e6086ff5d54f41a970d47fc5a
  Contract Address:       0x01d091b30a2d20ca2509579f8beae26934bfdc3725c0b497f50b353b7a3c636f
  Entry Point Selector:   __execute__
  Entry Point Type:       EXTERNAL
  Events:                 []
  Execution Resources
    L1 Gas:               18
    L2 Gas:               0
  Is Reverted:            false
  Messages:               []
  Result:                 success: array![array![].span()]
  Calls
    Call Type:            CALL
    Calldata:             ReportContext { config_digest: 0x441b0ab7fcd3923bd830e146e99ed90c4aebd19951eb6ed7b3713241aa8af, epoch_and_round: 2746113_u64, extra_hash: 0xf29c0193adc354752489f1a7af2f507d72a5e5b76cce705094d05d72e21ab5 }, 1716898684_u64, 0x304020100000000000000000000000000000000000000000000000000000000, array![10579338242_u128, 10580308946_u128, 10580390000_u128, 10581110000_u128], 214347425458270590264_u128, 1_u128, array![Signature { r: 0x6771e459d1e5563ec13af0ca40f04406ff4b70e6cc9a534dce12957f46c0f24, s: 0x36383aebe2151145a66dd7a87d9c885a862339e35d2ee0bd9df4075d17a8979, public_key: 0x2cb74dff29a13dd5d855159349ec92f943bacf0547ff3734e7d84a15d08cbc5 }, Signature { r: 0xb1a29e2cfed2f0a9d5f137845280bb6ce746f2f4b6a2dd05ec794171f4012, s: 0x1f85c957582717816bd2c910ac678caf007f6f84d71bc5a95f38de0b6435163, public_key: 0x4225d1c8ee8e451a25e30c10689ef898e11ccf5c0f68d0fc7876c47b318e946 }]
    Caller Address:       0x01d091b30a2d20ca2509579f8beae26934bfdc3725c0b497f50b353b7a3c636f
    Class Hash:           0x011be039724de97b52cd9ff391e36b4de91b6528d99dcd39b1161b3a64f87d65
    Contract Address:     0x0424ce41bea300e095e763d9fb4316af76c9da9c0fa926009f25b42b6f4ad04a
    Entry Point Selector: transmit
    Entry Point Type:     EXTERNAL
    Events
      Data:               [0x276a3f070, 0x6655cb7c, 0x304020100000000000000000000000000000000000000000000000000000000, 0x4, 0x27693e402, 0x276a2b3d2, 0x276a3f070, 0x276aeecf0, 0xb9eab07caffbd5538, 0x1, 0x441b0ab7fcd3923bd830e146e99ed90c4aebd19951eb6ed7b3713241aa8af, 0x29e701, 0x0]
      Keys:               [0x19e22f866f4c5aead2809bf160d2b29e921e335d899979732101c6f3c38ff81, 0xa36d, 0x1d091b30a2d20ca2509579f8beae26934bfdc3725c0b497f50b353b7a3c636f]
      Order:              0
    Execution Resources
      L1 Gas:             18
      L2 Gas:             0
    Is Reverted:          false
    Messages:             []
    Result:               success
    Calls:                []
Execution Resources
  L1 Data Gas:            640
  L1 Gas:                 32
  L2 Gas:                 0
Fee Transfer Invocation
  Call Type:              CALL
  Calldata:               ContractAddress(0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8), 4014902418114130240_u256
  Caller Address:         0x01d091b30a2d20ca2509579f8beae26934bfdc3725c0b497f50b353b7a3c636f
  Class Hash:             0x04ad3c1dc8413453db314497945b6903e1c766495a1e60492d44da9c2a986e4b
  Contract Address:       0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
  Entry Point Selector:   transfer
  Entry Point Type:       EXTERNAL
  Events
    Data:                 [0x1d091b30a2d20ca2509579f8beae26934bfdc3725c0b497f50b353b7a3c636f, 0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8, 0x37b7cc7a378cb140, 0x0]
    Keys:                 [0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9]
    Order:                0
  Execution Resources
    L1 Gas:               4
    L2 Gas:               0
  Is Reverted:            false
  Messages:               []
  Result:                 success: true
  Calls:                  []
Validate Invocation
  Call Type:              CALL
  Calldata:               array![Call { to: ContractAddress(0x424ce41bea300e095e763d9fb4316af76c9da9c0fa926009f25b42b6f4ad04a), selector: 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8, calldata: array![0x441b0ab7fcd3923bd830e146e99ed90c4aebd19951eb6ed7b3713241aa8af, 0x29e701, 0xf29c0193adc354752489f1a7af2f507d72a5e5b76cce705094d05d72e21ab5, 0x6655cb7c, 0x304020100000000000000000000000000000000000000000000000000000000, 0x4, 0x27693e402, 0x276a2b3d2, 0x276a3f070, 0x276aeecf0, 0xb9eab07caffbd5538, 0x1, 0x2, 0x6771e459d1e5563ec13af0ca40f04406ff4b70e6cc9a534dce12957f46c0f24, 0x36383aebe2151145a66dd7a87d9c885a862339e35d2ee0bd9df4075d17a8979, 0x2cb74dff29a13dd5d855159349ec92f943bacf0547ff3734e7d84a15d08cbc5, 0xb1a29e2cfed2f0a9d5f137845280bb6ce746f2f4b6a2dd05ec794171f4012, 0x1f85c957582717816bd2c910ac678caf007f6f84d71bc5a95f38de0b6435163, 0x4225d1c8ee8e451a25e30c10689ef898e11ccf5c0f68d0fc7876c47b318e946].span() }]
  Caller Address:         0x0000000000000000000000000000000000000000000000000000000000000000
  Class Hash:             0x066559c86e66214ba1bc5d6512f6411aa066493e6086ff5d54f41a970d47fc5a
  Contract Address:       0x01d091b30a2d20ca2509579f8beae26934bfdc3725c0b497f50b353b7a3c636f
  Entry Point Selector:   __validate__
  Entry Point Type:       EXTERNAL
  Events:                 []
  Execution Resources
    L1 Gas:               8
    L2 Gas:               0
  Is Reverted:            false
  Messages:               []
  Result:                 success: 0x56414c4944
  Calls:                  []
```

</details>
