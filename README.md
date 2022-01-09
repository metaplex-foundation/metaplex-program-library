# Metaplex Program Library 

Metaplex smart contracts and SDK.

## Metaplex Contracts


| Name                                 | Program                                                                                                                                                                                                                        | SDK                                                                                                                                                                                                                                | Integration Test                                                                                                                                                                                                                                                              |
| -----------                          | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------                              | ---                                                                                                                                                                                                                                | -----------                                                                                                                                                                                                                                                                   |
| [Candy Machine](./nft-candy-machine) | [![Program Candy Machine ](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-candy-machine.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-candy-machine.yml)     | [![SDK Metaplex](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-candy-machine.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-candy-machine.yml)         |                                                                                                                                                                                                                                                                               |
| [Token Vault](./token-vault)         | [![Program Token Vault](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-token-vault.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-token-vault.yml)             | [![SDK Token Vault](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-vault.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-vault.yml)             |                                                                                                                                                                                                                                                                               |
| [Token Entangler](./token-entangler) | [![Program Token Entangler](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-token-entangler.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-token-entangler.yml) | [![SDK Token Entangler](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-entangler.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-entangler.yml) |                                                                                                                                                                                                                                                                               |
| [Token Metadata](./token-metadata)   | [![Program Token Metadata ](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-token-metadata.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-token-metadata.yml)   | [![SDK Token Metadata](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-metadata.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-metadata.yml)    | [![Integration Token Metadata](https://github.com/metaplex-foundation/metaplex-program-library/actions/workflows/integration-token-metadata.yml/badge.svg)](https://github.com/metaplex-foundation/metaplex-program-library/actions/workflows/integration-token-metadata.yml) |
| [Auction](./auction)                 | [![Program Auction ](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-auction.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-auction.yml)                        | [![SDK Auction](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-auction.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-auction.yml)                         |                                                                                                                                                                                                                                                                               |
| [Auction House](./auction-house)     | [![Program Auction House ](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-auction-house.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-auction-house.yml)      | [![SDK Auction House](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-auction-house.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-auction-house.yml)       |                                                                                                                                                                                                                                                                               |
| [Metaplex](./metaplex)               | [![Program Metaplex ](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-metaplex.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-metaplex.yml)                     | [![SDK Metaplex](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-metaplex.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-metaplex.yml)                      |                                                                                                                                                                                                                                                                               |
| [NFT-Packs](./nft-packs)             | [![Program NFT-Packs ](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-nft-packs.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-nft-packs.yml)  |                                                                                                                                                                                                                                    |                                                                                                                                                                                                                                                                               |
| [Gumdrop](./gumdrop)                 | [![Program Gumdrop](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-gumdrop.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-gumdrop.yml)                         | [![SDK Gumdrop](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-gumdrop.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-gumdrop.yml)                         |                                                                                                                                                                                                                                                                               |

## Stability

[Stability 1 - Experimental](https://docs.metaplex.com/stability). Direct use of this library is not recommended in production or mainnet environments.

## Development

### Versioning and Publishing Packages

Smart contract SDK packages are versioned independently since a contract isn't necessarily
coupled to other contracts.

We use the following `(pre|post)(version|publish)` npm scripts to manage related checks,
tagging, commiting and pushing the version bump.

- `preversion`: ensures that the package builds and its tests pass
- `postversion`: adds and commits the version bump and adds a tag indicating package name and
  new version, i.e. `@metaplex-foundation/mp-core@v0.0.1`
- `prepublish`: ensures that the package builds and its tests pass again (just to be _really_
  sure)
- `postpublish`: pushes the committed change and new tag to github

In order to version and then publish a package just run the following commands from the folder
of the package you want to update:

- `npm version <patch|minor|major>`
- `npm publish`

As you note if version + publish succeeds the scripts end up pushing those updates to the
master branch. Therefore please ensure to be on and up to date `master` branch before running
them. Please **don't ever publish from another branch** but only from the main one with only
PR approved changes merged.

## Reporting security issues

To report a security issue, please follow the guidance on the [SECURITY](.github/SECURITY.md) page.
