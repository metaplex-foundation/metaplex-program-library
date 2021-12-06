# Metaplex Program Library 

Metaplex smart contracts and SDK.

## Metaplex Contracts

| Name                               | Program                                                                                                                                                                                                                      | SDK                                                                                                                                                                                                                             |
| -----------                        | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------                            | ---                                                                                                                                                                                                                             |
| [Token Vault](./token-vault)       | [![Program Token Vault](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-token-vault.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-token-vault.yml)           | [![SDK Token Vault](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-vault.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-vault.yml)          |
| [Token Metadata](./token-metadata) | [![Program Token Metadata ](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-token-metadata.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-token-metadata.yml) | [![SDK Token Metadata](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-metadata.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-token-metadata.yml) |
| [Auction](./auction)               | [![Program Auction ](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-auction.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-auction.yml)                      | [![SDK Auction](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-auction.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-auction.yml)                      |
| [Metaplex](./metaplex)             | [![Program Metaplex ](https://github.com/metaplex/metaplex-program-library/actions/workflows/program-metaplex.yml/badge.svg)](https://github.com/metaplex/teamplex/actions/workflows/program-metaplex.yml)                   | [![SDK Metaplex](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-metaplex.yml/badge.svg)](https://github.com/metaplex/metaplex-program-library/actions/workflows/sdk-metaplex.yml)                   |

## Development

### Versioning and Publishing Packages

We use [np](https://github.com/sindresorhus/np) to manage versioning and publishing packages in
order to ensure that the package builds and all tests pass.

You have to be on an up to date `master` branch in order to perform that step (`np` will verify
that).

NOTE: that packages are versioned indepenently since contracts aren't necessarily coupled to
other contracts.

As part of that step proper git tags are generated and pushed. A commit with a proper commit
message is generated as well and pushed.

### Steps

1. Make sure all changes are merged into the master branch
2. Checkout master
3. `cd` into the package `js` folder, i.e. `cd core/js`
4. Run `yarn np`
