# mpl-fixed-price-sale

This package contains the Metaplex Fixed Price Sale contract SDK code.

## API Docs

Find the [gumdrop API docs published here](https://metaplex-foundation.github.io/metaplex-program-library/docs/gumdrop/index.html).

## Installation

```shell
npm install @metaplex-foundation/mpl-fixed-price-sale --save
```

## Developing

In order to update the generated SDK when the rust contract was updated please run:

```
yarn api:gen
```

NOTE: at this point this only generates the IDL json file but later will generate TypeScript
definitions and SDK code as well, derived from that IDL.

## LICENSE

Apache v2.0

## Test

To run tests locally use

```
yarn amman:start
yarn build
yarn test
```
