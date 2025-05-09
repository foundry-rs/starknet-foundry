Set-StrictMode -Version Latest

$ErrorActionPreference = "Stop"

$DEVNET_INSTALL_DIR = Join-Path (git rev-parse --show-toplevel) "crates\sncast\tests\utils\devnet"

$DEVNET_REPO = "https://github.com/0xSpaceShard/starknet-devnet-rs.git"
$DEVNET_REV = "26292d1f92807090776b470f43b321f150f55ffd" # v0.4.0

cargo install --locked --git $DEVNET_REPO --rev $DEVNET_REV --root $DEVNET_INSTALL_DIR --force

Write-Host "All done!"

exit 0
