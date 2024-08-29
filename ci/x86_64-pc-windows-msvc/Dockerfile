ARG RUST_VERSION=1.80.0

FROM rust:$RUST_VERSION as builder

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV XWIN_ARCH=x86_64
ENV XWIN_INCLUDE_DEBUG_LIBS=true
ENV XWIN_INCLUDE_DEBUG_SYMBOLS=true

# C++ build dependencies
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install --no-install-recommends -y \
        clang \
        cmake \
        ninja-build && \
    apt-get remove -y --auto-remove && \
    rm -rf /var/lib/apt/lists/*

# Install Rust targets
RUN rustup target add x86_64-pc-windows-msvc && \
    rustup component add llvm-tools-preview

# cargo-xwin doesn't have an `init` command, so build an empty project instead
# to trigger the download.
RUN cargo install --locked cargo-xwin && \
    cargo new --bin foo && \
    cargo xwin build --manifest-path foo/Cargo.toml --target x86_64-pc-windows-msvc && \
    rm foo -r
