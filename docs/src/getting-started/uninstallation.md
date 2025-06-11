# Uninstallation
## Linux & macOS
Use the `asdf` docs to learn how to remove the Starknet Foundry plugin [here](https://asdf-vm.com/manage/plugins.html#remove)

To ensure Starknet Foundry is fully removed, run:

```bash
snforge --version
```

If the command is not found, the uninstallation was successful.

## Windows
### Remove Starknet Foundry Files
You can delete Starknet Foundry manually by navigating to:

```
📂 C:\Users\YourUsername\AppData\Local\Programs\snfoundry
```

Right-click the folder and select Delete.

### Remove Starknet Foundry from System PATH
If you added snfoundry/bin to your system PATH, you need to remove it:

For Windows 10 & 11:

1.	Open the Start Menu, search for “Environment Variables”, and open Edit the system environment variables.
2.	In the System Properties window, click Environment Variables.
3.	Under System variables (or User variables), find the Path entry and select Edit.
4.	Look for an entry similar to:

```
C:\Users\YourUsername\AppData\Local\Programs\snfoundry\bin
```

5.	Select it and click Delete, then OK to save changes.

### Uninstall Universal Sierra Compiler (Optional, if installed)
If you installed Universal Sierra Compiler, remove it as well:

```powershell
rm -r -Force "$env:LOCALAPPDATA\Programs\universal-sierra-compiler"
```

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