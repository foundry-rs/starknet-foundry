# Uninstallation
## Linux & macOS
Use the `asdf` docs to learn how to remove the Starknet Foundry plugin [here](https://asdf-vm.com/manage/plugins.html#remove)

To ensure Starknet Foundry is fully removed, run:

```bash
snforge --version
```

If the command is not found, the uninstallation was successful.


Also, remove its bin path from system Environment Variables, following the same steps as above.

### Uninstall Scarb (If No Longer Needed)
If you installed Scarb manually, remove it:

```powershell
rm -r -Force "$env:LOCALAPPDATA\Programs\scarb"
```

Then, remove scarb/bin from your system PATH using the steps mentioned earlier.

### Verify Uninstallation
To ensure Starknet Foundry is fully removed, open a Command Prompt or PowerShell and run:

```powershell
snforge --version
sncast --version
```

If you see “command not found”, the uninstallation was successful. 