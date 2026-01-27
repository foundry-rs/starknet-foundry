#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DATA_DIR="$PROJECT_ROOT/crates/sncast/tests/data/"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Ledger Integration Tests ===${NC}"

# 1. Validation Checks
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}Error: Docker is not running${NC}" ; exit 1
fi

APP_FILE="$TEST_DATA_DIR/ledger-app/nanox#strk#0.25.13.elf"
if [ ! -f "$APP_FILE" ]; then
    echo -e "${RED}Error: Ledger app binary not found at $APP_FILE${NC}"
    echo -e "${YELLOW}Please download or build the Starknet Ledger app binary${NC}"
    echo "Expected: app-starknet version 2.4.2 for Nano X (SDK 22.10.0)"
    echo "Source: https://github.com/LedgerHQ/app-starknet/releases"
    exit 1
fi

# Validate the binary is a valid ELF file
FILE_INFO=$(file "$APP_FILE")
if ! echo "$FILE_INFO" | grep -q "ELF.*ARM"; then
    echo -e "${RED}Error: Ledger app binary is not a valid ARM ELF executable at $APP_FILE${NC}"
    echo "File info: $FILE_INFO"
    exit 1
fi

if echo "$FILE_INFO" | grep -q "too large section header"; then
    echo -e "${RED}Error: Ledger app binary is corrupted at $APP_FILE${NC}"
    echo -e "${YELLOW}The file has an invalid section header offset${NC}"
    echo ""
    echo "File info: $FILE_INFO"
    echo ""
    echo "To fix this:"
    echo "  1. Download the correct binary from: https://github.com/LedgerHQ/app-starknet/releases"
    echo "  2. Or build it from source: https://github.com/LedgerHQ/app-starknet"
    echo "  3. Place the binary at: $APP_FILE"
    echo ""
    echo "Expected: app-starknet version 2.4.2 for Nano X (SDK 22.10.0)"
    exit 1
fi

TEST_FILTER="${1:-ledger}"
CARGO_FLAGS="${2:---nocapture --ignored --test-threads=1}"
SKIP_TESTS="${3:-}"

echo -e "${BLUE}Test Configuration:${NC}"
echo "  Filter: $TEST_FILTER"
echo "  Flags: $CARGO_FLAGS"
echo "  Skip: $SKIP_TESTS"
echo ""
echo -e "${YELLOW}Available filters:${NC}"
echo "  ledger                    - Run all Ledger tests (unit + network)"
echo "  ledger::ledger_tests      - Run only unit tests (ledger.rs)"
echo "  ledger_network_tests      - Run only network integration tests"
echo "  test_name                 - Run specific test"
echo ""

# 2. Define Directories
DOCKER_HOME_DIR="$PROJECT_ROOT/target-docker-home"
mkdir -p "$DOCKER_HOME_DIR"

# 3. FIX PERMISSIONS
echo -e "${YELLOW}Fixing volume permissions...${NC}"
docker run --rm \
    -v ledger_build_cache:/data \
    ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:latest \
    chown -R "$(id -u):$(id -g)" /data

echo -e "${GREEN}Running tests in Docker...${NC}"

# 4. The Run Command
# Use -it only if running in a terminal
TTY_FLAG=""
if [ -t 0 ]; then
    TTY_FLAG="-it"
fi

docker run --rm $TTY_FLAG \
    --user "$(id -u):$(id -g)" \
    -p 5001-5003:5001-5003 \
    -p 6001-6002:6001-6002 \
    -p 5055:5055 \
    -v "$PROJECT_ROOT:/workspace" \
    -v ledger_build_cache:/workspace/target \
    -v "$DOCKER_HOME_DIR:/tmp/home" \
    -v "$HOME/.cargo/registry:/tmp/home/.cargo/registry" \
    -w /workspace \
    -e HOME=/tmp/home \
    -e RUSTUP_HOME=/tmp/home/.rustup \
    -e CARGO_HOME=/tmp/home/.cargo \
    -e CARGO_TARGET_DIR=/workspace/target \
    -e RUST_BACKTRACE=1 \
    -e TEST_FILTER="$TEST_FILTER" \
    -e CARGO_FLAGS="$CARGO_FLAGS" \
    -e SKIP_TESTS="$SKIP_TESTS" \
    ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:latest \
    bash -c "
        set -e
        export PATH=\"/tmp/home/.cargo/bin:\$PATH\"

        # Install Rust
        if ! command -v cargo &> /dev/null; then
            echo 'Installing Rust...'
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --no-modify-path
        fi
        
        # Ensure default toolchain is configured
        rustup default stable
        
        # Install Starknet Devnet
        if ! command -v starknet-devnet &> /dev/null; then
            echo 'Installing starknet-devnet...'
            cargo install starknet-devnet --locked
        fi
        
        echo 'Starting Speculos instances...'
        speculos --model nanox --display headless --api-port 5001 --apdu-port 9001 /workspace/crates/sncast/tests/data/ledger-app/nanox#strk#0.25.13.elf &
        SPECULOS_PID_1=\$!
        
        speculos --model nanox --display headless --api-port 5002 --apdu-port 9002 /workspace/crates/sncast/tests/data/ledger-app/nanox#strk#0.25.13.elf &
        SPECULOS_PID_2=\$!
        
        speculos --model nanox --display headless --api-port 5003 --apdu-port 9003 /workspace/crates/sncast/tests/data/ledger-app/nanox#strk#0.25.13.elf &
        SPECULOS_PID_3=\$!
        
        speculos --model nanox --display headless --api-port 6001 --apdu-port 9004 /workspace/crates/sncast/tests/data/ledger-app/nanox#strk#0.25.13.elf &
        SPECULOS_PID_4=\$!
        
        speculos --model nanox --display headless --api-port 6002 --apdu-port 9005 /workspace/crates/sncast/tests/data/ledger-app/nanox#strk#0.25.13.elf &
        SPECULOS_PID_5=\$!

        speculos --model nanox --display headless --api-port 4001 --apdu-port 9006 /workspace/crates/sncast/tests/data/ledger-app/nanox#strk#0.25.13.elf &
        SPECULOS_PID_6=\$!
        
        speculos --model nanox --display headless --api-port 4002 --apdu-port 9007 /workspace/crates/sncast/tests/data/ledger-app/nanox#strk#0.25.13.elf &
        SPECULOS_PID_7=\$!
                
        speculos --model nanox --display headless --api-port 4003 --apdu-port 9008 /workspace/crates/sncast/tests/data/ledger-app/nanox#strk#0.25.13.elf &
        SPECULOS_PID_8=\$!
        
        cleanup() {
            echo 'Stopping services...'
            kill \$SPECULOS_PID_1 \$SPECULOS_PID_2 \$SPECULOS_PID_3 \$SPECULOS_PID_4 \$SPECULOS_PID_5 \$SPECULOS_PID_6 \$SPECULOS_PID_7 \$SPECULOS_PID_8 2>/dev/null || true
        }
        trap cleanup EXIT

        echo 'Waiting for Speculos instances to be ready...'
        for port in 5001 5002 5003 6001 6002 4001 4002 4003; do
            for i in {1..30}; do
                if curl -s http://localhost:\$port/ > /dev/null 2>&1; then
                    echo \"  Port \$port is ready\"
                    break
                fi
                if [ \$i -eq 30 ]; then
                    echo \"Error: Speculos on port \$port failed to start\"
                    exit 1
                fi
                sleep 0.5
            done
        done
        echo 'All Speculos instances ready!'

        echo 'Running tests...'
        if cargo test -p sncast --test main \"\$TEST_FILTER\" -- \$CARGO_FLAGS \$SKIP_TESTS; then
             echo -e '${GREEN}All tests passed.${NC}'
        else
             echo -e '${RED}Tests failed.${NC}'
        fi
        
        echo -e '${YELLOW}Press Enter to stop services and cleanup...${NC}'
        read -r
    "