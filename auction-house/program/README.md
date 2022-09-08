---
title: Auction House
---

## Background

To know more about the Auction House, see https://docs.metaplex.com/auction-house/definition

## Running the tests

To run the tests we need to build the token-metadata first, the steps are as follows:
- Navigate to the `metaplex-program-library/token-metadata/program` directory first.
- Run `cargo build-bpf --bpf-out-dir ../../target/deploy/` in your terminal.
- Once you run the builds you should see a `target/deploy` directory in your root folder, that would contain the `mpl-token-metadata.so ` file.
- Navigate to the `metaplex-program-library/auction-house/program` direstory and run 
```clear && RUST_LOG=debug cargo test-bpf --bpf-out-dir ../../target/deploy/ 2>&1 | grep -v CounterPoint```,
and you should see the tests running.