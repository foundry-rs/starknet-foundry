# Set strict mode
Set-StrictMode -Version Latest

# Stop the script on error
$ErrorActionPreference = "Stop"

# Get the path of the top-level Git directory
$DEVNET_INSTALL_DIR = Join-Path (git rev-parse --show-toplevel) "crates\sncast\tests\utils\devnet"

# Set the DEVNET repository and revision
$DEVNET_REPO = "https://github.com/0xSpaceShard/starknet-devnet-rs.git"
$DEVNET_REV = "ef789b700770fa27a2fc057b3d1c610771be27d9"

# Perform cargo install
cargo install --locked --git $DEVNET_REPO --rev $DEVNET_REV --root $DEVNET_INSTALL_DIR --force

# Output completion message
Write-Host "All done!"

# Exit with success
exit 0
