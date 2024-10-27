#!/usr/bin/env bash
set -e

# Configuration
RPC_URL="http://localhost:5050"
COMPILED_CONTRACT_PATH="./build/main.json"

# This ensures that sncast and scarb are installed
if ! command -v sncast &> /dev/null; then
  echo "sncast not found. Please install it first."
  exit 1
fi

if ! command -v scarb &> /dev/null; then
  echo "scarb not found. Please install it first."
  exit 1
fi

# This Deploys the contract
echo "Deploying the contract using sncast..."
DEPLOY_RESPONSE=$(sncast deploy --url "$RPC_URL" --contract "$COMPILED_CONTRACT_PATH" --inputs 0x0)

# This extracts the contract address from the deployment response
CONTRACT_ADDRESS=$(echo "$DEPLOY_RESPONSE" | grep -oP '(?<="contract_address": ")[^"]*')

if [ -z "$CONTRACT_ADDRESS" ]; then
  echo "Failed to deploy the contract. Please do well to check the output above for errors."
  exit 1
fi

echo "Contract deployed successfully! Contract Address: $CONTRACT_ADDRESS"

# This saves deployment details to a file for reference
echo "Saving deployment details to deployment_details.txt..."
echo "Contract Address: $CONTRACT_ADDRESS" > deployment_details.txt # This line creates a new file named deployment_details.txt and writes the "Contract Address" to it 
echo "RPC URL: $RPC_URL" >> deployment_details.txt # This line appends the "RPC URL" to the file
echo "Compiled Contract Path: $COMPILED_CONTRACT_PATH" >> deployment_details.txt # This line appends the "Compiled Contract Path" to the file
echo "Deployment Output: $DEPLOY_RESPONSE" >> deployment_details.txt # This line appends the "Deployment Output" to the file

echo "Deployment complete! You can now find the details in deployment_details.txt."
