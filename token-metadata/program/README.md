---
title: Token Metadata Program
---

## Testing
```sh
cargo test-bpf --bpf-out-dir ../../test-programs/
```
## Building
```sh
cargo build-bpf --bpf-out-dir ../../test-programs/
```

## Source

The Token Metadata Program's source is available on
[github](https://github.com/metaplex-foundation/metaplex-program-library)

There is also an example Rust client located at
[github](https://github.com/metaplex-foundation/metaplex-program-library/tree/master/token-metadata/test/src/main.rs)
that can be perused for learning and run if desired with `cargo run --bin metaplex-token-metadata-test-client`. It allows testing out a variety of scenarios.

## Interface

The on-chain Token Metadata program is written in Rust and available on crates.io as
[mpl-token-metadata](https://crates.io/crates/mpl-token-metadata) and
[docs.rs](https://docs.rs/mpl-token-metadata).


## Documentation

Full docs for this program can be found [here](https://docs.metaplex.com/programs/token-metadata/).