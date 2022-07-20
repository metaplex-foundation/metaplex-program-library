# mpl-hydra

This package contains the hydra contract SDK code.

## API Docs

Find the
[hydra API docs published here](https://metaplex-foundation.github.io/metaplex-program-library/docs/hydra/index.html).

## Developing

In order to update the generated SDK when the rust contract was updated please run:

```
yarn gen:api
```

and then update the wrapper code and tests.

## LICENSE

Apache v2.0

## Test

To run tests locally use

```
yarn amman:start
yarn build
yarn test
```
