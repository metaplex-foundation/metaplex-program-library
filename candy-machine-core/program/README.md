# Metaplex Candy Machine Core (a.k.a. Candy Machine V3)

> 🛑 **DO NOT USE IN PRODUCTION**: This repository contain a proof-of-concept.

## Overview

The Metaplex Protocol's `Candy Machine` is the leading minting and distribution program for fair NFT collection launches on Solana. It allows creators to bring their asset metadata on-chain and provides a mechanism to create (mint) NFTs from the on-chain configuration &mdash; in both deterministic and non-deterministic (random) way, like a traditional candy machine.

The latest iteration of the `Candy Machine` program retains the core functionality of minting an NFT only, while any access control configuration is now done by the new [`Candy Guard`](https://github.com/metaplex-foundation/candy-guard) program.

The `Candy Machine` program is responsible for:
- config line (asset) management: configuration of how many assets are available and their metadata information;
- index generation and selection;
- NFT minting (token creation).

### Why separating mint logic and access control?

Previous versions of the `Candy Machine` included access controls to limit who can mint, when minting is allowed and other settings related to the mint (e.g., price) as part of its minting procedure. While the combination of the *mint logic* with *access controls* works, this design makes it complex to add more features or to remove undesired ones.

Creating a clear separation provides a modular and flexible architechture to add and remove access control settings, whithout the risks and complexities of breaking the minting logic. Each access control logic can be implemented in isolation and users can choose to enable/disable individual ones.

### Who can mint from a Candy Machine?

Each `Candy Machine` account has two authorities associated: an `authority` and `mint_authority`. The `authority` has permission to manage the configuration of the candy machine. This include adding config lines (assets), updating its settings, changing authority/mint authorities and closing/withdrawing the account.

The `mint_authority` is the only address that is able to mint from a candy machine &mdash; the only requirement being that there are NFTs to be minted from it. The `authority` of the candy machine can delegate the `mint_authority` to another address, either a wallet or PDA. This separation allows maximum flexibility and enables composability, since other programs can be set as `mint_authority` and provide additional features on top of the candy machine. An example of this is the [`Candy Guard`](https://github.com/metaplex-foundation/candy-guard) program, which provides access control guards for the candy machine.

## Account

The `Candy Machine` configuration is stored in a single account, which includes settings that control the behaviour of the candy machine and metadata information for the NFTs minted through it. The account data is represented by the [`CandyMachine`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine.rs) struct, which include references to auxiliary structs [`Creator`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs), [`ConfigLineSettings`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs) and [`HiddenSettings`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs).

| Field                       | Offset | Size  | Description                  |
| --------------------------- | ------ | ----- | ---------------------------- |
| &mdash;                     | 0      | 8     | Anchor account discriminator.
| `features`                  | 8      | 8     | `u64` field to be used as a binary flag to support future features while maintaing backwards compatibility. |
| `authority`                 | 16     | 32    | `PubKey` of the authority address that controls the candy machine. |
| `mint_authority`            | 48     | 32    | `PubKey` of the address allowed to mint from the candy machine. |
| `collection_mint`           | 80     | 32    | `PubKey` of the collection NFT; each NFT minted from the candy machine will be part of this collection. |
| `items_redeemed`            | 112    | 8     | Number of NFTs minted. |
| `data`                      |        |       |  [`CandyMachineData`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs)         |
| - `items_available`         | 120    | 8     | Total number of NFTs available. |
| - `symbol`                  | 128    | 14    | `string` representing the token symbol: `length` (4 bytes) + `symbol` (10 bytes). |
| - `seller_fee_basis_points` | 142    | 2     | Royalties percentage awarded to creators (value between 0 and 1000). |
| - `max_supply`              | 144    | 8     | Indicates how many copies (editions) of an NFT can be created after it is minted; this is usually set to `0`. |
| - `is_mutable`              | 152    | 1     | Indicates whether the minted NFT is mutable or not. |
| - `creators`                | 153    | ~     | An array of [`Creator`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs#L29) and their share of the royalties; this array is limited to 5 creators. **Note:** since the `creators` field is an array of variable length, we cannot guarantee the byte position of any field that follows (Notice the tilde ~ in the fields below). Each creator contains the following fields: |
| -- `address`                | ~      | 32    | The public key of the creator |
| -- `verified`               | ~      | 1     | The public key of the creator |
| -- `share`                  | ~      | 1     | The public key of the creator |
| - `config_line_settings`    | ~      | 1     | (optional) [`ConfigLineSettings`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs#L51)                             |
| -- `prefix_name`            | ~      | 36    | `string` representing the common part of the name of NFTs. |
| -- `name_length`            | ~      | 4     | `u32` specifying the number of bytes for the remaining part of the name. |
| -- `prefix_uri`             | ~      | 204   | `string` representing the common part of the URI of NFTs. |
| -- `uri_length`             | ~      | 4     | `u32` specifying the number of bytes for the remaining part of the URI. |
| -- `is_sequential`          | ~      | 1     | Indicates whether the mint index generation is sequential or not. |
| - `hidden_settings`         | ~      | 1     | (optional) [`HiddenSettings`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs#L40) |
| -- `name`                   | ~      | 36    | `string` representing the name of NFTs. |
| -- `uri`                    | ~      | 204   | `uri` for the metadata of NFTs.         |
| -- `hash`                   | ~      | 32    | `string` representing the hash value of the file that contain the mapping of (mint index, NFT metadata). |
| *hidden section*            | 850    | ~    | (optional) Hidden data section to avoid unnecessary deserialisation. This section of the account is not represented by structs and data is store/retrieved using byte offsets. The hidden data section is not present when `hiddenSettings` are used, since there is no need to store config line settings. |
| - *items*</div>             | 850    | 4    | Number of NFTs (items) added to the candy machine; eventually this will be the same as `items_available`. |
| - *config lines*</div>      | 854    | ~    | A sequence of name and uri pairs representing each NFT; the length of these are determined by `name_length + uri_length`; there will `items_available * (name + uri)` pairs in total. |
| - *byte mask*</div>         | ~      | ~    | A byte section of length equal to `(items_available / 8) + 1` with binary flag to indicate which config lines have been added. |
| - *mint indices*</div>      | ~      | ~    | A sequence of `u32` values representing the available mint indices; the usable indices are determined by: valid indices start at the mint number (`items_redeemed`) if `is_sequential` is `true`; valid mint indices start from offset 0 until the offset determined by `items_available - items_redeemed`. |
