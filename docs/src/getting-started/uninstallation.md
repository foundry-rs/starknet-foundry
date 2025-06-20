# Uninstallation
## Linux & macOS
Use the `asdf` docs to learn how to remove the Starknet Foundry plugin [here](https://asdf-vm.com/manage/plugins.html#remove)

To ensure Starknet Foundry is fully removed, run:

```bash
snforge --version
```

If the command is not found, the uninstallation was successful.

### Verify Uninstallation
To ensure Starknet Foundry is fully removed, open a Command Prompt or PowerShell and run:

```powershell
snforge --version
sncast --version
```

If you see “command not found”, the uninstallation was successful. 