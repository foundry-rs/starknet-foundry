#!/usr/bin/env bash
set -e

# Configuration
RPC_URL="http://localhost:5050"
COMPILED_CONTRACT_PATH="./build/main.json"

# This ensures that sncast and scarb are installed
if ! command -v sncast &> /dev/null; then
  echo "sncast is not found. Please install it."
  exit 1
fi

if ! command -v scarb &> /dev/null; then
  echo "sncast is not found. Please install it."
  exit 1
fi

# This Deploys the contract
echo "Deploying the contract using sncast..."
DEPLOY_RESPONSE=$(sncast deploy --url "$RPC_URL" --contract "$COMPILED_CONTRACT_PATH" --inputs 0x0)

