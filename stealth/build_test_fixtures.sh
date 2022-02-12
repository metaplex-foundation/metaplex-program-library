#!/usr/bin/env bash

set -e
set -x

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# build token metadata
cd "$SCRIPT_DIR/../token-metadata/program"
cargo build-bpf --dump

# copy workspace output to top-level
mkdir -p ../../target/deploy
cp ../target/deploy/mpl_token_metadata.so ../../target/deploy

# build stealth
cd "$SCRIPT_DIR/program"
cargo build-bpf --dump

# build stealth escrow
cd "$SCRIPT_DIR/escrow"
cargo build-bpf --dump
