Set-StrictMode -Version Latest

$ErrorActionPreference = "Stop"

$DEVNET_INSTALL_DIR = Join-Path (git rev-parse --show-toplevel) "crates\sncast\tests\utils\devnet"

$DEVNET_REPO = "https://github.com/0xSpaceShard/starknet-devnet-rs.git"
$DEVNET_REV = "ef789b700770fa27a2fc057b3d1c610771be27d9"

cargo install --locked --git $DEVNET_REPO --rev $DEVNET_REV --root $DEVNET_INSTALL_DIR --force

Write-Host "All done!"

exit 0
