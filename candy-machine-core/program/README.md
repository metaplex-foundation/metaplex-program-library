# Metaplex Candy Machine Core (a.k.a. Candy Machine V3)

> ⚠️ **Candy Machine V3 is currently experimental and has not been formally audited. Use in production
> at your own risk.**

## Overview

The Metaplex Protocol's `Candy Machine` is the leading minting and distribution program for fair NFT
collection launches on Solana. It allows creators to bring their asset metadata on-chain and
provides a mechanism to create (mint) NFTs from the on-chain configuration &mdash; in both
deterministic and non-deterministic (random) way, like a traditional candy machine.

The latest iteration of the `Candy Machine` program retains the core functionality of minting an NFT
only, while any access control configuration is now done by the new
[`Candy Guard`](https://github.com/metaplex-foundation/mpl-candy-guard) program.

The `Candy Machine` program is responsible for:

- config line (asset) management: configuration of how many assets are available and their metadata
  information;
- index generation and selection;
- NFT minting (token creation).

### Why separating mint logic and access control?

Previous versions of the `Candy Machine` included access controls to limit who can mint, when
minting is allowed and other settings related to the mint (e.g., price) as part of its minting
procedure. While the combination of the _mint logic_ with _access controls_ works, this design makes
it complex to add more features or to remove undesired ones.

Creating a clear separation provides a modular and flexible architechture to add and remove access
control settings, whithout the risks and complexities of breaking the minting logic. Each access
control logic can be implemented in isolation and users can choose to enable/disable individual
ones.

### Who can mint from a Candy Machine?

Each `Candy Machine` account has two authorities associated: an `authority` and `mint_authority`.
The `authority` has permission to manage the configuration of the candy machine. This include adding
config lines (assets), updating its settings, changing authority/mint authorities and
closing/withdrawing the account.

The `mint_authority` is the only address that is able to mint from a candy machine &mdash; the only
requirement being that there are NFTs to be minted from it. The `authority` of the candy machine can
delegate the `mint_authority` to another address, either a wallet or PDA. This separation allows
maximum flexibility and enables composability, since other programs can be set as `mint_authority`
and provide additional features on top of the candy machine. An example of this is the
[`Candy Guard`](https://github.com/metaplex-foundation/mpl-candy-guard) program, which provides
access control guards for the candy machine.

## Account

The `Candy Machine` configuration is stored in a single account, which includes settings that
control the behaviour of the candy machine and metadata information for the NFTs minted through it.
The account data is represented by the
[`CandyMachine`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine.rs)
struct, which include references to auxiliary structs
[`Creator`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs),
[`ConfigLineSettings`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs)
and
[`HiddenSettings`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs).

| Field                       | Offset | Size | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| --------------------------- | ------ | ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| &mdash;                     | 0      | 8    | Anchor account discriminator.                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `features`                  | 8      | 8    | `u64` field to be used as a binary flag to support future features while maintaing backwards compatibility.                                                                                                                                                                                                                                                                                                                                                                            |
| `authority`                 | 16     | 32   | `PubKey` of the authority address that controls the candy machine.                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `mint_authority`            | 48     | 32   | `PubKey` of the address allowed to mint from the candy machine.                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `collection_mint`           | 80     | 32   | `PubKey` of the collection NFT; each NFT minted from the candy machine will be part of this collection.                                                                                                                                                                                                                                                                                                                                                                                |
| `items_redeemed`            | 112    | 8    | Number of NFTs minted.                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `data`                      |        |      | [`CandyMachineData`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs)                                                                                                                                                                                                                                                                                                          |
| - `items_available`         | 120    | 8    | Total number of NFTs available.                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| - `symbol`                  | 128    | 14   | `string` representing the token symbol: `length` (4 bytes) + `symbol` (10 bytes).                                                                                                                                                                                                                                                                                                                                                                                                      |
| - `seller_fee_basis_points` | 142    | 2    | Royalties percentage awarded to creators (value between 0 and 1000).                                                                                                                                                                                                                                                                                                                                                                                                                   |
| - `max_supply`              | 144    | 8    | Indicates how many copies (editions) of an NFT can be created after it is minted; this is usually set to `0`.                                                                                                                                                                                                                                                                                                                                                                          |
| - `is_mutable`              | 152    | 1    | Indicates whether the minted NFT is mutable or not.                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| - `creators`                | 153    | ~    | An array of [`Creator`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs#L29) and their share of the royalties; this array is limited to 5 creators. **Note:** since the `creators` field is an array of variable length, we cannot guarantee the byte position of any field that follows (Notice the tilde ~ in the fields below). Each creator contains the following fields: |
| -- `address`                | ~      | 32   | The public key of the creator                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| -- `verified`               | ~      | 1    | The public key of the creator                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| -- `share`                  | ~      | 1    | The public key of the creator                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| - `config_line_settings`    | ~      | 1    | (optional) [`ConfigLineSettings`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs#L51)                                                                                                                                                                                                                                                                                         |
| -- `prefix_name`            | ~      | 36   | `string` representing the common part of the name of NFTs.                                                                                                                                                                                                                                                                                                                                                                                                                             |
| -- `name_length`            | ~      | 4    | `u32` specifying the number of bytes for the remaining part of the name.                                                                                                                                                                                                                                                                                                                                                                                                               |
| -- `prefix_uri`             | ~      | 204  | `string` representing the common part of the URI of NFTs.                                                                                                                                                                                                                                                                                                                                                                                                                              |
| -- `uri_length`             | ~      | 4    | `u32` specifying the number of bytes for the remaining part of the URI.                                                                                                                                                                                                                                                                                                                                                                                                                |
| -- `is_sequential`          | ~      | 1    | Indicates whether the mint index generation is sequential or not.                                                                                                                                                                                                                                                                                                                                                                                                                      |
| - `hidden_settings`         | ~      | 1    | (optional) [`HiddenSettings`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine_data.rs#L40)                                                                                                                                                                                                                                                                                             |
| -- `name`                   | ~      | 36   | `string` representing the name of NFTs.                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| -- `uri`                    | ~      | 204  | `uri` for the metadata of NFTs.                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| -- `hash`                   | ~      | 32   | `string` representing the hash value of the file that contain the mapping of (mint index, NFT metadata).                                                                                                                                                                                                                                                                                                                                                                               |
| _hidden section_            | 850    | ~    | (optional) Hidden data section to avoid unnecessary deserialisation. This section of the account is not represented by structs and data is store/retrieved using byte offsets. The hidden data section is not present when `hiddenSettings` are used, since there is no need to store config line settings.                                                                                                                                                                            |
| - _items_</div>             | 850    | 4    | Number of NFTs (items) added to the candy machine; eventually this will be the same as `items_available`.                                                                                                                                                                                                                                                                                                                                                                              |
| - _config lines_</div>      | 854    | ~    | A sequence of name and uri pairs representing each NFT; the length of these are determined by `name_length + uri_length`; there will `items_available * (name + uri)` pairs in total.                                                                                                                                                                                                                                                                                                  |
| - _byte mask_</div>         | ~      | ~    | A byte section of length equal to `(items_available / 8) + 1` with binary flags to indicate which config lines have been added.                                                                                                                                                                                                                                                                                                                                                        |
| - _mint indices_</div>      | ~      | ~    | A sequence of `u32` values representing the available mint indices; the usable indices are determined by: valid indices start at the mint number (`items_redeemed`) if `is_sequential` is `true`; otherwise, valid mint indices start from offset 0 until the offset determined by `items_available - items_redeemed`.                                                                                                                                                                 |

## Instructions

### 📄 `add_config_lines`

This instruction adds config lines to the hidden data section of the account. It can only be used if
the candy machine has `config_line_settings`.

<details>
  <summary>Accounts</summary>

| Name            | Writable | Signer | Description                                |
| --------------- | :------: | :----: | ------------------------------------------ |
| `candy_machine` |    ✅    |        | The `CandyMachine` account.                |
| `authority`     |          |   ✅   | Public key of the candy machine authority. |

</details>

<details>
  <summary>Arguments</summary>

| Argument                      | Offset | Size | Description               |
| ----------------------------- | ------ | ---- | ------------------------- |
| `index`                       | 0      | 4    | Index from which the lines will be added. |
| `config_lines`                | 4      | ~    | Array of [`ConfigLine`](https://github.com/metaplex-foundation/metaplex-program-library/blob/febo/candy-machine-core/candy-machine-core/program/src/state/candy_machine.rs#L33) objects representing the lines to be added. |
</details>

### 📄 `initialize`

This instruction creates and initializes a new `CandyMachine` account. It requires that the
CandyMachine account has been created with the expected size before executing this instruction.

<details>
  <summary>Accounts</summary>

| Name                          | Writable | Signer | Description                                                          |
| ----------------------------- | :------: | :----: | -------------------------------------------------------------------- |
| `candy_machine`               |    ✅    |        | The `CandyMachine` account.                                          |
| `authority_pda`               |    ✅    |        | Authority PDA key (seeds `["candy_machine", candy_machine pubkey]`). |
| `authority`                   |          |        | Public key of the candy machine authority.                           |
| `payer`                       |          |   ✅   | Payer of the transaction.                                            |
| `collection_metadata`         |          |        | Metadata account of the collection.                                  |
| `collection_mint`             |          |        | Mint account of the collection.                                      |
| `collection_master_edition`   |          |        | Master Edition account of the collection.                            |
| `collection_update_authority` |    ✅    |   ✅   | Update authority of the collection.                                  |
| `collection_authority_record` |    ✅    |        | Authority Record PDA of the collection.                              |
| `token_metadata_program`      |          |        | Metaplex `TokenMetadata` program ID.                                 |
| `system_program`              |          |        | `SystemProgram` account.                                             |
| `rent`                        |          |        | `Rent` account.                                                      |

</details>

<details>
  <summary>Arguments</summary>

| Argument                      | Offset | Size | Description               |
| ----------------------------- | ------ | ---- | ------------------------- |
| `data`                        | 0      | ~    | `CandyMachineData` object. |
</details>

### 📄 `mint`

This instruction mints an NFT from the Candy Machine. Only the mint authority is able to mint from
the Candy Machine.

<details>
  <summary>Accounts</summary>

| Name                          | Writable | Signer | Description                                                                               |
| ----------------------------- | :------: | :----: | ----------------------------------------------------------------------------------------- |
| `candy_machine`               |    ✅    |        | The `CandyMachine` account.                                                               |
| `authority_pda`               |    ✅    |        | Authority PDA key (seeds `["candy_machine", candy_machine pubkey]`).                      |
| `mint_authority`              |          |   ✅   | Public key of the candy machine mint authority.                                           |
| `payer`                       |    ✅    |   ✅   | Payer of the transaction.                                                                 |
| `nft_mint`                    |    ✅    |        | Mint account for the NFT. The account should be created before executing the instruction. |
| `nft_mint_authority`          |          |   ✅   | Mint authority of the NFT.                                                                |
| `nft_metadata`                |    ✅    |        | Metadata account of the NFT.                                                              |
| `nft_master_edition`          |    ✅    |        | Master Edition account of the NFT.                                                        |
| `collection_authority_record` |          |        | Authority Record PDA of the collection.                                                   |
| `collection_mint`             |          |        | Mint account of the collection.                                                           |
| `collection_metadata`         |    ✅    |        | Metadata account of the collection.                                                       |
| `collection_master_edition`   |          |        | Master Edition account of the collection.                                                 |
| `collection_update_authority` |          |        | Update authority of the collection.                                                       |
| `token_metadata_program`      |          |        | Metaplex `TokenMetadata` program ID.                                                      |
| `token_program`               |          |        | `spl-token` program ID.                                                                   |
| `system_program`              |          |        | `SystemProgram` account.                                                                  |
| `rent`                        |          |        | `Rent` account.                                                                           |

</details>

<details>
  <summary>Arguments</summary>

None.
</details>

### 📄 `set_authority`

This instruction changes the authority of the candy machine. Note that this operation is
irreversible, once you change the authority of the Candy Machine, the current authority will lose
the right to operate it.

<details>
  <summary>Accounts</summary>

| Name            | Writable | Signer | Description                                |
| --------------- | :------: | :----: | ------------------------------------------ |
| `candy_machine` |    ✅    |        | The `CandyMachine` account.                |
| `authority`     |          |   ✅   | Public key of the candy machine authority. |

</details>

<details>
  <summary>Arguments</summary>

| Argument                      | Offset | Size | Description               |
| ----------------------------- | ------ | ---- | ------------------------- |
| `new_authority`               | 0      | 32    | Public key of the new authority. |
</details>

### 📄 `set_collection`

This instruction sets the collection to be used by the Candy Machine. The collection can only be
changed if no NFTs have been minted.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description                                                          |
| --------------------------------- | :------: | :----: | -------------------------------------------------------------------- |
| `candy_machine`                   |    ✅    |        | The `CandyMachine` account.                                          |
| `authority`                       |          |   ✅   | Public key of the candy machine authority.                           |
| `authority_pda`                   |    ✅    |        | Authority PDA key (seeds `["candy_machine", candy_machine pubkey]`). |
| `payer`                           |          |   ✅   | Payer of the transaction.                                            |
| `collection_mint`                 |          |        | Mint account of the current collection.                              |
| `collection_metadata`             |          |        | Metadata account of the current collection.                          |
| `collection_authority_record`     |    ✅    |        | Authority Record PDA of the current collection.                      |
| `new_collection_update_authority` |    ✅    |   ✅   | Authority Record PDA of the new collection.                          |
| `new_collection_metadata`         |          |        | Metadata account of the new collection.                              |
| `new_collection_mint`             |          |        | Mint account of the new collection.                                  |
| `new_collection_master_edition`   |          |        | Master Edition account of the new collection.                        |
| `new_collection_authority_record` |    ✅    |        | Authority Record PDA of the new collection.                          |
| `token_metadata_program`          |          |        | Metaplex `TokenMetadata` program ID.                                 |
| `system_program`                  |          |        | `SystemProgram` account.                                             |
| `rent`                            |          |        | `Rent` account.                                                      |

</details>

<details>
  <summary>Arguments</summary>

None.
</details>

### 📄 `set_mint_authority`

This instruction changes the mint authority of the Candy Machine. Note that this operation is
irreversible, once you change the mint authority of the Candy Machine, the current mint authority
will lose the right to mint from the Candy Machine.

<details>
  <summary>Accounts</summary>

| Name             | Writable | Signer | Description                                |
| ---------------- | :------: | :----: | ------------------------------------------ |
| `candy_machine`  |    ✅    |        | The `CandyMachine` account.                |
| `authority`      |          |   ✅   | Public key of the candy machine authority. |
| `mint_authority` |          |   ✅   | Public key of the new mint authority.      |

</details>

<details>
  <summary>Arguments</summary>

None.
</details>

### 📄 `update`

This instruction updates the configuration of the Candy Machine. There are restrictions on which
configuration can be updated:

- `items_available`: can only be updated when `hidden_settings` are used.
- `hidden_settings`: it is not possible to switch to `hidden_settings` if the number of
  `items_available` is greater than `0`; it is not possuble to swith from `hidden_settings` to
  `config_line_settings`.
- `name_length` and `uri_length` in `config_line_settings`: can only be updated with values that are
  smaller that current values used.
- `is_sequential`: can only be changed is the number of `items_redemmed` is equal to `0`.

<details>
  <summary>Accounts</summary>

| Name            | Writable | Signer | Description                                |
| --------------- | :------: | :----: | ------------------------------------------ |
| `candy_machine` |    ✅    |        | The `CandyMachine` account.                |
| `authority`     |          |   ✅   | Public key of the candy machine authority. |

</details>

<details>
  <summary>Arguments</summary>

| Argument                      | Offset | Size | Description               |
| ----------------------------- | ------ | ---- | ------------------------- |
| `data`                        | 0      | ~    | `CandyMachineData` object. |
</details>

### 📄 `withdraw`

This instruction withdraws the rent lamports from the account and closes it. After executing this
instruction, the Candy Machine will not be operational.

<details>
  <summary>Accounts</summary>

| Name            | Writable | Signer | Description                                |
| --------------- | :------: | :----: | ------------------------------------------ |
| `candy_machine` |    ✅    |        | The `CandyMachine` account.                |
| `authority`     |    ✅    |   ✅   | Public key of the candy machine authority. |

</details>

<details>
  <summary>Arguments</summary>

None.
</details>

## Features

Main improvements over the previous Candy Machine program.

### Account space utilization

It is now possible to define a pattern to store the `name` and `uri` configuration in the format of
`prefix_name + name` and `prefix_uri + uri`, where both `prefix_name` and `prefix_uri` are shared
among all config lines. This provides account space saving, since there is no need to store repeated
bytes in the account, leading to the possibility of creating larger Candy Machines and reducing the
cost of deployment.

Instead of storing full URIs &mdash; e.g.,
`https://arweave.net/yFoNLhe6cBK-wj0n_Wu-XuX7DC75VbMsNKwVbRSz4iQ?ext=png` &mdash; for each config
line, the `prefix_uri` is set to `https://arweave.net/` and each config line only stores the
different values in the `uri` space. This also applies to the `prefix_name` and `name` pair.

When a storage with deterministic URI generation &mdash; e.g., AWS S3 and Shadow Drive &mdash; is
used, a significant space saving can be achieved by using replacement patterns in the `prefix_name`
and `prefix_uri`, leaving each individual `name` and `uri` empty. In this case, the only space
needed is to store the index representing the id for the random mint index generation.

A prefix_uri can include `$ID$` or `$ID+1$` patterns, which are automatically substituted for the
`mint index` or `mint index + 1` to generate a valid uri:

- `https://shdw-drive.genesysgo.net/DCG6qThfZE8xbM72RoFRLwRSrhNVjeWE1gVPPCGvLYSS/$ID$.png` gets
  expanded to `https://shdw-drive.genesysgo.net/DCG6qThfZE8xbM72RoFRLwRSrhNVjeWE1gVPPCGvLYSS/0.png`
  when the first NFT is minted.

This also applied to the `prefix_name`: `My NFT #$ID+1$` gets expanded to `My NFT #1` when the fist
NFT is minted.

### Hidden settings with "automatic" reveal

Hidden settings are the most space efficient way to create a `Candy Machine` since no config lines
are stored on the account. At the same time, it is necessary to run an update metadata procedure
&mdash; dubbed **reveal** &mdash; after the mint to set each individual name and URI on the metadata
of minted NFTs.

When a storage with deterministic URI generation is used, the **reveal** is not required as one can
use the same `$ID$`or `$ID+1$` replacement patterns in the **name** and **uri** fields of hidden
settings.

The difference between using config lines with patterns and hidden settings with patterns is that in
the former, the index generation for the mint is **random** while in the later the index generation
is **sequential**.

> **Note** While the use of deterministic URIs saves you work in terms of not requiring to run an
> update the metadata on each NFT, it would be possible to determine the URI of an NFT before it is
> minted. In order to avoid the files to be publicly accessible ahead of time, a placeholder image
> with the same name can be used instead. Therefore, the trade-off is between running an update
> metadata on each NFT or updating the images.

### Random Index Generation

Currently the random index generation uses a sequential procedure to find the next available mint
index. While this procedure works for most cases, it is not efficient (in terms of compute units)
and it can reach the limit of compute units on large Candy Machine deploys.

The new Candy Machine uses an improved procedure that consumes a fixed amount of compute units
regardless of the number of items and, at the same time, shuffles the values to improve their
unpredictability.
