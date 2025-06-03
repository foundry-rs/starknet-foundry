Set-StrictMode -Version Latest

$ErrorActionPreference = "Stop"

$DEVNET_INSTALL_DIR = Join-Path (git rev-parse --show-toplevel) "crates\sncast\tests\utils\devnet"

$DEVNET_REPO = "https://github.com/0xSpaceShard/starknet-devnet.git"
$DEVNET_REV = "e0613b1028013dfd7ee8239d677e3a544fd808c8" # v0.4.2

cargo install --locked --git $DEVNET_REPO --rev $DEVNET_REV --root $DEVNET_INSTALL_DIR --force

Write-Host "All done!"

exit 0
