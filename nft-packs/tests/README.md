# Metaplex-tests crate

Before run tests you have to move *.so program files on which we depends.

More specifically there should be shared objects(*.so files) for [Metaplex token metadata program](https://github.com/metaplex-foundation/metaplex/tree/master/rust/token-metadata/program), [Metaplex program](https://github.com/metaplex-foundation/metaplex/tree/master/rust/metaplex/program) and [randomness oracle program](https://github.com/atticlab/randomness-oracle/tree/main/program).

To get shared objects of that program you have to build it with command `cargo build-bpf` and this command will show you path to that file.

To run tests just do `cargo test-bpf`
