# Metaplex Token Metadata

The Token Metadata program is one of the most important programs when dealing with NFTs on the Solana blockchain. Its main goal is to **attach additional data to Fungible or Non-Fungible Tokens** on Solana.

## Building

From the root directory of the repository:
```sh
cargo build-bpf --bpf-out-dir ../../test-programs/
```

## Testing (BPF)
From the root directory of the repository:
```sh
cargo test-bpf --bpf-out-dir ../../test-programs/
```

## Testing (TypeScript)
Integration tests are available using [Amman](https://github.com/metaplex-foundation/amman).

After building the program, go to the folder `../js` and run:
```
yarn install
```

On a separate terminal, start Amman from the `../js` folder:
```
yarn amman:start
```

Back to your main terminal on the `../js` folder, run:
```
yarn build && yarn test
```

## Source

The Token Metadata Program's source is available on
[github](https://github.com/metaplex-foundation/metaplex-program-library)

## Interface

The on-chain Token Metadata program is written in Rust and available on crates.io as
[mpl-token-metadata](https://crates.io/crates/mpl-token-metadata) and
[docs.rs](https://docs.rs/mpl-token-metadata).

## Documentation

Full docs for this program can be found [here](https://docs.metaplex.com/programs/token-metadata/).
