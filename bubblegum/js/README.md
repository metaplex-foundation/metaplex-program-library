# mpl-bubblegum

This package contains the Metaplex Bubblegum program JS SDK code.

## HOW IT WORKS
[Program Overview](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/README.md)

## API Docs

Find the [bubblegum API docs published here](https://metaplex-foundation.github.io/metaplex-program-library/docs/bubblegum/index.html).

## Installation

```shell
npm install @metaplex-foundation/mpl-bubblegum --save
```

## Developing

In order to update the generated SDK when the Rust contract was updated please run:
```
yarn api:gen
```
and then update the wrapper code and tests.

## Running tests

```shell
## Build the latest bubblegum
pushd ../program/
cargo build-bpf
popd
yarn run start-validator

yarn test
```

## LICENSE

Apache v2.0
