# Dockerfile for cross-compilation with LLVM 19
FROM ubuntu:22.04

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Set temp directory variables
ENV TMPDIR=/tmp
ENV TMP=/tmp

ENV TEMP=/tmp

# Install prerequisites and add the official LLVM apt repository for Ubuntu 22.04 (jammy)
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        gnupg \
        wget \
        software-properties-common \
        lsb-release \
    && wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add - \
    && echo "deb http://apt.llvm.org/jammy/ llvm-toolchain-jammy-19 main" > /etc/apt/sources.list.d/llvm-repo.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        llvm-19 \
        llvm-19-dev \
        llvm-19-runtime \
        clang-19 \
        clang-tools-19 \
        lld-19 \
        libpolly-19-dev \
        libmlir-19-dev \
        mlir-19-tools \
        libomp-19-dev \
    && rm -rf /var/lib/apt/lists/*

# Ensure LLVM 19 unsuffixed tools are available first in PATH
ENV PATH=/usr/lib/llvm-19/bin:${PATH}
ENV LD_LIBRARY_PATH=/usr/lib/llvm-19/lib

# Export prefixes expected by Rust llvm/mlir crates
ENV MLIR_SYS_190_PREFIX=/usr/lib/llvm-19
ENV LLVM_SYS_191_PREFIX=/usr/lib/llvm-19
ENV TABLEGEN_190_PREFIX=/usr/lib/llvm-19

# Additional environment variables for bindgen/clang to find headers and libraries
ENV CPPFLAGS="-I/usr/lib/llvm-19/include"
ENV BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/llvm-19/include"
ENV LDFLAGS="-L/usr/lib/llvm-19/lib"

# MLIR TableGen specific environment variables
ENV MLIR_TBLGEN_EXE=/usr/lib/llvm-19/bin/mlir-tblgen
ENV TBLGEN_INCLUDES="-I/usr/lib/llvm-19/include/mlir -I/usr/lib/llvm-19/include"

# Install additional development packages for cross-compilation
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        zlib1g-dev \
        libzstd-dev \
        libxml2-dev \
        gcc-aarch64-linux-gnu \
        g++-aarch64-linux-gnu \
        libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

# Create cargo home directory with proper permissions
RUN mkdir -p /root/.cargo/bin && \
    chmod -R 755 /root/.cargo

# Create a build directory that's accessible to all users
RUN mkdir -p /build && \
    chmod -R 777 /build && \
    mkdir -p /usr/local/cargo-target && \
    chmod -R 777 /usr/local/cargo-target

ENV CARGO_TARGET_DIR=/usr/local/cargo-target

# Ensure the filesystem supports executable permissions
RUN mount | grep "exec\|noexec" || echo "No mount restrictions found"

# Set the working directory
WORKDIR /project

# Default command
CMD ["/bin/bash"]
