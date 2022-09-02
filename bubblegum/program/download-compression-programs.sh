#!/usr/bin/env bash

MPL_ROOT=$(git rev-parse --show-toplevel)
mkdir -p $MPL_ROOT/target/deploy

mkdir solana_program_library || true
curl -LkSs https://api.github.com/repos/solana-labs/solana-program-library/tarball | tar -xz --strip-components=1 -C ./solana_program_library
tar -zxf -C /solana_program_library solana-program-library.tar.gz
pushd solana_program_library/account-compression/programs/account-compression
  cargo build-bpf --bpf-out-dir ./here
  mv ./here/spl_compression.so $MPL_ROOT/target/deploy/GRoLLzvxpxxu2PGNJMMeZPyMxjAUH9pKqxGXV9DGiceU.so
popd

pushd solana_program_library/account-compression/programs/wrapper
  cargo build-bpf --bpf-out-dir ./here
  mv ./here/wrapper.so $MPL_ROOT/target/deploy/WRAPYChf58WFCnyjXKJHtrPgzKXgHp6MD9aVDqJBbGh.so
popd

rm -rf solana_program_library
