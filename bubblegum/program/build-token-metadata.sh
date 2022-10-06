#!/bin/bash

# Simple script to build token-metdata and move it to the root
# `target/deploy` folder to load for banks client tests. This is
# mean to be invoked via `build.rs` where the current working dir
# is already set properly.

set -e

cd ../../token-metadata
mkdir -p ../target/deploy
cargo build-bpf --bpf-out-dir ../target/deploy
