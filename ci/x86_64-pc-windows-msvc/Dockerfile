# Copyright 2024 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

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
