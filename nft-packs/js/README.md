# mpl-nft-packs

This package contains the token vault contract SDK code.

## API Docs

Find the [nft-packs API docs published here](https://metaplex-foundation.github.io/metaplex-program-library/docs/nft-packs/index.html).

## Installation

```shell
npm install @metaplex-foundation/mpl-nft-packs --save
```

This will use the new auto-generated API. If you want to use the old deprecated API, you can use the following:

```shell
npm install @metaplex-foundation/mpl-nft-packs@^1.2 --save
```

Alternatively, if you'd like to start using the new API but still have access to the deprecated one, you may use the following:

```sh
npm install @metaplex-foundation/mpl-nft-packs@~2.0 --save
```

And use the deprecated API explicitly like so:

```ts
import { deprecated } from "@metaplex-foundation/mpl-nft-packs";
```

## Developing

In order to update the generated SDK when the rust contract was updated please run:

```
yarn gen:api
```

and then update the wrapper code and tests.

## LICENSE

Apache v2.0
