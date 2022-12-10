#!/usr/bin/env bash

# Exit immediately on error.
set -e

MPL_ROOT=$(git rev-parse --show-toplevel)
mkdir -p $MPL_ROOT/test-programs

mkdir -p solana_program_library
curl -LkSs https://api.github.com/repos/solana-labs/solana-program-library/tarball | tar -xz --strip-components=1 -C ./solana_program_library

pushd solana_program_library/account-compression/programs/account-compression
  cargo build-bpf --bpf-out-dir $MPL_ROOT/test-programs
popd

pushd solana_program_library/account-compression/programs/noop
  cargo build-bpf --bpf-out-dir $MPL_ROOT/test-programs
popd

rm -rf solana_program_library
