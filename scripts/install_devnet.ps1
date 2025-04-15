Set-StrictMode -Version Latest

$ErrorActionPreference = "Stop"

$DEVNET_INSTALL_DIR = Join-Path (git rev-parse --show-toplevel) "crates\sncast\tests\utils\devnet"

$DEVNET_REPO = "https://github.com/0xSpaceShard/starknet-devnet-rs.git"
$DEVNET_REV = "fc5a2753a2eedcc27eed7a4fae3ecac08c2ca1b4"

cargo install --locked --git $DEVNET_REPO --rev $DEVNET_REV --root $DEVNET_INSTALL_DIR --force

Write-Host "All done!"

exit 0
