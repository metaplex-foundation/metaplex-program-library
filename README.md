# Metaplex Program Library

Metaplex smart contracts and SDK.

## Metaplex Contracts

| Name                                   | Program                                                                       | SDK                                                                           | Integration Test                                                                   |
| :------------------------------------- | :---------------------------------------------------------------------------- | :---------------------------------------------------------------------------  | :--------------------------------------------------------------------------------- |
| [Candy Machine](./candy-machine)       | [![Program Candy Machine][p-candy-machine-svg]][p-candy-machine-yml]          | [![SDK Candy Machine][sdk-candy-machine-svg]][sdk-candy-machine-yml]          |                                                                                    |
| [Token Entangler](./token-entangler)   | [![Program Token Entangler][p-token-entangler-svg]][p-token-entangler-yml]    | [![SDK Token Entangler][sdk-token-entangler-svg]][sdk-token-entangler-yml]    |                                                                                    |
| [Token Metadata](./token-metadata)     | [![Program Token Metadata][p-token-metadata-svg]][p-token-metadata-yml]       | [![SDK Token Metadata][sdk-token-metadata-svg]][sdk-token-metadata-yml]       |                                                                                    |
| [Auction House](./auction-house)       | [![Program Auction House][p-auction-house-svg]][p-auction-house-yml]          | [![SDK Auction House][sdk-auction-house-svg]][sdk-auction-house-yml]          |                                                                                    |
| [NFT-Packs](./nft-packs)               | [![Program NFT-Packs][p-nft-packs-svg]][p-nft-packs-yml]                      |                                                                               |                                                                                    |
| [Gumdrop](./gumdrop)                   | [![Program Gumdrop][p-gumdrop-svg]][p-gumdrop-yml]                            | [![SDK Gumdrop][sdk-gumdrop-svg]][sdk-gumdrop-yml]                            |                                                                                    |
| [Fixed price sale](./fixed-price-sale) | [![Program Fixed Price Sale][p-fixed-price-sale-svg]][p-fixed-price-sale-yml] | [![SDK Fixed Price Sale][sdk-fixed-price-sale-svg]][sdk-fixed-price-sale-yml] | [![Integration Fixed Price Sale][i-fixed-price-sale-svg]][i-fixed-price-sale-yml]  |


## Development

### Setting up Rust Tests

Run the `build.sh` script with the name of the program to build the shared object and put it in a directory called `test-programs` in the root of the project.

E.g.:

```bash
./build.sh auction-house
```

Running with `all` builds all programs.


### Versioning and Publishing Packages

Smart contract SDK packages are versioned independently since a contract isn't necessarily coupled
to other contracts.

We use the following `(pre|post)(version|publish)` npm scripts to manage related checks, tagging,
commiting and pushing the version bump.

- `preversion`: ensures that the package builds and its tests pass
- `postversion`: adds and commits the version bump and adds a tag indicating package name and new
  version, i.e. `@metaplex-foundation/mp-core@v0.0.1`
- `prepublishOnly`: ensures that the package builds and its tests pass again (just to be _really_ sure)
- `postpublish`: pushes the committed change and new tag to github

In order to version and then publish a package just run the following commands from the folder of
the package you want to update:

- `npm version <patch|minor|major>`
- `npm publish`

As you note if version + publish succeeds the scripts end up pushing those updates to the master
branch. Therefore please ensure to be on and up to date `master` branch before running them. Please
**don't ever publish from another branch** but only from the main one with only PR approved changes
merged.

### Rust Crates

| Package          | Link                                            | Version                                                        |
| :--------------- | :---------------------------------------------- | :------------------------------------------------------------- |
| Candy Machine    | [mpl-candy-machine][mpl-candy-machine-crate]    | [![Crate][mpl-candy-machine-img]][mpl-candy-machine-crate]     |
| Token Metadata   | [mpl-token-metadata][mpl-token-metadata-crate]  | [![Crate][mpl-token-metadata-img]][mpl-token-metadata-crate]   |
| Auction House    | [mpl-auction-house][mpl-auction-house-crate]    | [![Crate][mpl-auction-house-img]][mpl-auction-house-crate]     |
| Testing Utils    | [mpl-testing-utils][mpl-testing-utils-crate]    | [![Crate][mpl-testing-utils-img]][mpl-testing-utils-crate]     |

### Npm Packages

| Package        | Link                                          | Version                                                    |
| :------------- | :-------------------------------------------- | :--------------------------------------------------------- |
| Candy Machine  | [mpl-candy-machine][mpl-candy-machine-npm]    | [![NPM][mpl-candy-machine-nimg]][mpl-candy-machine-npm]    |
| Token Metadata | [mpl-token-metadata][mpl-token-metadata-npm]  | [![NPM][mpl-token-metadata-nimg]][mpl-token-metadata-npm]  |
| Core           | [mpl-core][mpl-core-npm]                      | [![NPM][mpl-core-nimg]][mpl-core-npm]                      |
| Auction House  | [mpl-auction-house][mpl-auction-house-npm]    | [![NPM][mpl-auction-house-nimg]][mpl-auction-house-npm]    |

## Reporting security issues

To report a security issue, please follow the guidance on the [SECURITY](.github/SECURITY.md) page.

## License

The Rust/Cargo programs are licensed under the “Apache-style” [Metaplex(TM) NFT Open Source License](metaplex-nft-license) and the JS/TS client libraries are licensed under either the [MIT](mit-license) or the [Apache](apache-license) licenses.


<!-- ===================================== -->
<!-- Links for badges and such shown above -->
<!-- Please add any links you add to the   -->
<!-- readme here instead of inlining them  -->
<!-- ===================================== -->

<!-- Program Badges -->
[p-candy-machine-yml]:https://github.com/metaplex/teamplex/actions/workflows/program-candy-machine.yml
[p-candy-machine-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/program-candy-machine.yml/badge.svg
[p-token-entangler-yml]:https://github.com/metaplex/teamplex/actions/workflows/program-token-entangler.yml
[p-token-entangler-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/program-token-entangler.yml/badge.svg
[p-token-metadata-yml]:https://github.com/metaplex/teamplex/actions/workflows/program-token-metadata.yml
[p-token-metadata-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/program-token-metadata.yml/badge.svg
[p-auction-house-yml]:https://github.com/metaplex/teamplex/actions/workflows/program-auction-house.yml
[p-auction-house-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/program-auction-house.yml/badge.svg
[p-nft-packs-yml]:https://github.com/metaplex/metaplex-program-library/actions/workflows/program-nft-packs.yml
[p-nft-packs-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/program-nft-packs.yml/badge.svg
[p-gumdrop-yml]:https://github.com/metaplex/teamplex/actions/workflows/program-gumdrop.yml
[p-gumdrop-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/program-gumdrop.yml/badge.svg
[p-fixed-price-sale-yml]:https://github.com/metaplex/metaplex-program-library/actions/workflows/program-fixed-price-sale.yml
[p-fixed-price-sale-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/program-fixed-price-sale.yml/badge.svg

<!-- SDK Badges  -->
[sdk-candy-machine-yml]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-candy-machine.yml
[sdk-candy-machine-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-candy-machine.yml/badge.svg
[sdk-token-entangler-yml]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-entangler.yml
[sdk-token-entangler-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-entangler.yml/badge.svg
[sdk-token-metadata-yml]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-metadata.yml
[sdk-token-metadata-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-metadata.yml/badge.svg
[sdk-auction-house-yml]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-auction-house.yml
[sdk-auction-house-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-auction-house.yml/badge.svg
[sdk-gumdrop-yml]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-gumdrop.yml
[sdk-gumdrop-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-gumdrop.yml/badge.svg
[sdk-fixed-price-sale-yml]:https://github.com/metaplex-foundation/metaplex-program-library/actions/workflows/sdk-fixed-price-sale.yml
[sdk-fixed-price-sale-svg]:https://github.com/metaplex-foundation/metaplex-program-library/actions/workflows/sdk-fixed-price-sale.yml/badge.svg
[sdk-nft-packs-yml]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-nft-packs.yml
[sdk-nft-packs-svg]:https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-nft-packs.yml/badge.svg

<!-- Integration Badges -->
[i-fixed-price-sale-svg]:https://github.com/metaplex-foundation/metaplex-program-library/actions/workflows/integration-fixed-price-sale.yml/badge.svg
[i-fixed-price-sale-yml]:https://github.com/metaplex-foundation/metaplex-program-library/actions/workflows/integration-fixed-price-sale.yml

<!-- Crates -->
[mpl-candy-machine-crate]:https://crates.io/crates/mpl-candy-machine
[mpl-token-metadata-crate]:https://crates.io/crates/mpl-token-metadata
[mpl-auction-house-crate]:https://crates.io/crates/mpl-auction-house
[mpl-testing-utils-crate]:https://crates.io/crates/mpl-testing-utils
[mpl-candy-machine-img]:https://img.shields.io/crates/v/mpl-candy-machine
[mpl-token-metadata-img]:https://img.shields.io/crates/v/mpl-token-metadata
[mpl-auction-house-img]:https://img.shields.io/crates/v/mpl-auction-house
[mpl-testing-utils-img]:https://img.shields.io/crates/v/mpl-testing-utils

<!-- NPM Packages -->
[mpl-candy-machine-npm]:https://www.npmjs.com/package/@metaplex-foundation/mpl-candy-machine
[mpl-token-metadata-npm]:https://www.npmjs.com/package/@metaplex-foundation/mpl-token-metadata
[mpl-core-npm]:https://www.npmjs.com/package/@metaplex-foundation/mpl-core
[mpl-auction-house-npm]:https://www.npmjs.com/package/@metaplex-foundation/mpl-auction-house
[mpl-candy-machine-nimg]:https://img.shields.io/npm/v/@metaplex-foundation/mpl-candy-machine
[mpl-token-metadata-nimg]:https://img.shields.io/npm/v/@metaplex-foundation/mpl-token-metadata
[mpl-core-nimg]:https://img.shields.io/npm/v/@metaplex-foundation/mpl-core
[mpl-auction-house-nimg]:https://img.shields.io/npm/v/@metaplex-foundation/mpl-auction-house

<!-- Licenses -->
[metaplex-nft-license]:  https://github.com/metaplex-foundation/metaplex-program-library/blob/master/LICENSE
[apache-license]: https://www.apache.org/licenses/LICENSE-2.0.txt
[mit-license]: https://www.mit.edu/~amini/LICENSE.md
