#!/usr/bin/env bash

set -euxo pipefail

LLVM_VERSION="19.1.6"
LLVM_TARGET="x86_64-linux-gnu-ubuntu-22.04"
LLVM_URL="https://github.com/llvm/llvm-project/releases/download/llvmorg-${LLVM_VERSION}/clang+llvm-${LLVM_VERSION}-${LLVM_TARGET}.tar.xz"
LLVM_DIR="/opt/llvm-19"

echo "Installing LLVM ${LLVM_VERSION} for cross-compilation..."

# Download LLVM
wget -O llvm.tar.xz "${LLVM_URL}"

# Extract and install
tar -xf llvm.tar.xz
mv "clang+llvm-${LLVM_VERSION}-${LLVM_TARGET}" "${LLVM_DIR}"
rm llvm.tar.xz

# Set permissions
chmod -R 755 "${LLVM_DIR}"

echo "LLVM ${LLVM_VERSION} installed successfully at ${LLVM_DIR}"
echo "Setting environment variables..."

# Export environment variables for the build
export MLIR_SYS_190_PREFIX="${LLVM_DIR}"
export LLVM_SYS_191_PREFIX="${LLVM_DIR}"
export TABLEGEN_190_PREFIX="${LLVM_DIR}"
export PATH="${LLVM_DIR}/bin:$PATH"
export LD_LIBRARY_PATH="${LLVM_DIR}/lib:${LD_LIBRARY_PATH:-}"

echo "Environment configured:"
echo "MLIR_SYS_190_PREFIX=${MLIR_SYS_190_PREFIX}"
echo "LLVM_SYS_191_PREFIX=${LLVM_SYS_191_PREFIX}"
echo "TABLEGEN_190_PREFIX=${TABLEGEN_190_PREFIX}"
echo "PATH=${PATH}"
echo "LD_LIBRARY_PATH=${LD_LIBRARY_PATH}"

# Verify installation
"${LLVM_DIR}/bin/llvm-config" --version
"${LLVM_DIR}/bin/clang" --version
