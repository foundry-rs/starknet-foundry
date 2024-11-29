# Set strict mode
Set-StrictMode -Version Latest

# Stop the script on error
$ErrorActionPreference = "Stop"

# Get the path of the top-level Git directory
$DEVNET_INSTALL_DIR = Join-Path (git rev-parse --show-toplevel) "crates\sncast\tests\utils\devnet"

# Set the DEVNET repository and revision
$DEVNET_REPO = "https://github.com/0xSpaceShard/starknet-devnet-rs.git"
$DEVNET_TAG = "0.2.2"

# Perform cargo install
cargo install --locked --git $DEVNET_REPO --tag $DEVNET_TAG --root $DEVNET_INSTALL_DIR --force

# Output completion message
Write-Host "All done!"

# Exit with success
exit 0