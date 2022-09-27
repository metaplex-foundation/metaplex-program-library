#!/usr/bin/env bash

# Exit immediately on error.
set -e

MPL_ROOT=$(git rev-parse --show-toplevel)
mkdir -p $MPL_ROOT/target/deploy

mkdir solana_program_library || true
curl -LkSs https://api.github.com/repos/solana-labs/solana-program-library/tarball/feeda6a6 | tar -xz --strip-components=1 -C ./solana_program_library

pushd solana_program_library/account-compression/programs/account-compression
  cargo build-bpf --bpf-out-dir ./here
  mv ./here/spl_account_compression.so $MPL_ROOT/target/deploy/GRoLLzvxpxxu2PGNJMMeZPyMxjAUH9pKqxGXV9DGiceU.so
popd

pushd solana_program_library/account-compression/programs/noop
  cargo build-bpf --bpf-out-dir ./here
  mv ./here/spl_noop.so $MPL_ROOT/target/deploy/WRAPYChf58WFCnyjXKJHtrPgzKXgHp6MD9aVDqJBbGh.so
popd

rm -rf solana_program_library
