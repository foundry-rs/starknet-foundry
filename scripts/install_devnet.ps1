Set-StrictMode -Version Latest

$ErrorActionPreference = "Stop"

$DEVNET_INSTALL_DIR = Join-Path (git rev-parse --show-toplevel) "crates\sncast\tests\utils\devnet"

$DEVNET_REPO = "https://github.com/0xSpaceShard/starknet-devnet-rs.git"
$DEVNET_REV = "4ee3267d46099f11e12bc9751d08ef31a4d48512" # v0.4.0

cargo install --locked --git $DEVNET_REPO --rev $DEVNET_REV --root $DEVNET_INSTALL_DIR --force

Write-Host "All done!"

exit 0
