# Creating And Deploying Accounts

Account is required to perform interactions with Starknet (only calls can be done without it). Starknet Foundry cast supports
entire account management flow with the `sncast account create` and `sncast account deploy` commands.

Difference between those two commands is that the first one creates account information (private key, address and more)
and the second one deploys it to the network. After deployment, account can be used to interact with Starknet.

Do the following to start interacting with the Starknet:

- create account with the `sncast account create` command

    ```shell
    $ sncast \
      --url http://127.0.0.1:5050
      --network testnet \
      account create \
      --name some-name
      
    Account successfully created. Prefund generated address with at least 432300000000 tokens. It is good to send more in the case of higher demand, max_fee * 2 = 864600000000
    command: account create
    max_fee: 0x64a7168300
    address: 0x7a949e83b243068d0cbedd8d5b8b32fafea66c54de23c40e68b126b5c845b61
    ```

    You can also pass common `--accounts-file` argument with a path to (existing or not existing) file where you want to save account info.
    
    For a detailed CLI description, see [account create command reference](../appendix/cast/account/create.md).


- prefund generated address with tokens
  
    You can do it both by sending tokens from another starknet account or by bridging them with [StarkGate](https://starkgate.starknet.io/).


- deploy account with the `sncast account deploy` command

    ```shell
    $ sncast \
      --url http://127.0.0.1:5050
      --network testnet \
      account deploy
      --name some-name \
      --max-fee 864600000000
    
    command: account deploy
    transaction_hash: 0x20b20896ce63371ef015d66b4dd89bf18c5510a840b4a85a43a983caa6e2579
    ```
  
    Note that you don't have to pass `url` and `network` parameters if `add-profile` flag
    was set in the `account create` command. Just pass `profile` argument with the account name.
    
    For a detailed CLI description, see [account deploy command reference](../appendix/cast/account/deploy.md).
