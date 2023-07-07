# Bubblegum - Compressed Metaplex NFTs

> ⚠️ **Bubblegum is currently experimental and has not been formally audited. Use in production
> at your own risk.**

## Testing
```bash
cargo build
cargo test-bpf --bpf-out-dir ../../test-programs
```


## Overview

`Bubblegum` is the Metaplex Protocol program for creating and interacting with compressed Metaplex NFTs.  Compressed NFTs are secured on-chain using Merkle trees.

With Bubblegum you can:
* Create a tree
* Delegate authority for a tree.
* Mint a compressed NFT to a tree.
* Verify/unverify creators
* Verify/unverify membership of an NFT to a Metaplex Verified Collection.
* Transfer ownership of an NFT.
* Delegate authority for an NFT.
* Burn an NFT.
* Redeem an NFT and decompress it into an uncompressed Metaplex NFT.

## Background

Compressed NFTs differ from uncompressed NFTs in where their state (metadata, owner, etc.) is stored.  For uncompressed NFTs, all state data is stored in on-chain accounts.  This tends to be expensive at scale.  Compressed NFTs save space by encoding the state data into a Merkle tree.  This means that the detailed account data is not stored on-chain, but in data stores managed by RPC providers.

Compressed NFTs are secured on-chain by the hashing of the state data when it is added to the Merkle tree.  The Merkle root is a hash that cryptographically secures the state data for all of the leaves (NFTs) contained in the tree.

In the unlikely scenario that all RPC providers were to lose their data stores, the off-chain state of compressed NFTs could be recovered by replaying transactions (provided that the given tree was started from the beginning).

Compressed NFTs can also be losslessly decompressed into uncompressed Metaplex NFTs.  Decompression will cost rent for the Metadata and Master Edition `token-metadata` program accounts that need to be created.

## Basic operation

### Creating and minting to a tree

Anyone can create a tree using `create_tree` and then they are the tree owner.  They can also delegate authority to another wallet.

### Merkle proofs

After an NFT is minted, for any operations that modify the NFT, Merkle proofs must be provided with the instruction to validate the Merkle tree changes.  Bubblegum is an Anchor program and makes use of the remaining accounts feature for this purpose.  Merkle proofs are added as remaining account `Pubkey`s that are 32-byte Keccak256 hash values that represent the nodes from the Merkle tree that are required to calculate a new Merkle root.

### Creator verification

Creators are specified in a creators array in the metadata passed into the `mint_v1` instruction.  All creators for which `verified` is set to true in the creators array at the time of mint must be a signer of the transaction or the mint will fail.  Beyond the signers specified in the `MintV1` Account validation structure, the `mint_v1` instruction uses remaining accounts to optionally look for additional signing creators.  This does not conflict with the Merkle proofs requirement because proofs are not required for minting.

Beyond verifying creators at the time of mint, there are `verify_creator` and `unverify_creator` instructions that can be used on existing Compressed NFTs.

### Collection verification

Note that there is no such thing as compressed Verified Collections.  Collections are still NFTs created in the realm of Metadata and Master Edition `token-metadata` accounts. There are instructions to `verify_collection` and `unverify_collection`, as well as a `set_and_verify_collection` instruction for the case where the collection was set during the mint.  All of these require either the true Collection Authority to be a a signer, or a legacy `token-metadata` delegated Collection Authority to be a signer along with providing the Collection Authority Record PDA.

`mint_to_collection_v1` is an instruction that can be used to mint a compressed NFT and verify collection at the same time.  Also note that `decompress_v1` is now able to decompress assets with verified collection.

See the Metaplex documentation on [`Certified Collections`](https://docs.metaplex.com/programs/token-metadata/certified-collections) for more information on verifying collections.

### Transfer ownership, delegate authority, and burn an NFT.

Compressed NFTs support transferring ownership, delegating authority, and burning the NFT.  See the [Instructions](##Instructions) section below for details.

### Redeem an NFT and decompress it into an uncompressed Metaplex NFT

Redeeming an NFT removes the leaf from the Merkle tree and creates a voucher PDA account.  The voucher account can be sent to the `decompress_v1` instruction to decompress the NFT into an uncompressed Metaplex NFT.  As mentioned above this will cost rent for the Metadata and Master Edition `token-metadata` accounts that are created during the decompression process.  Note that after a compressed NFT is redeemed but before it is decompressed, the process can be reversed using `cancel_redeem`.  This puts the compressed NFT back into the Merkle tree.

## Accounts

### 📄 `tree_authority`

The `tree_authority` PDA account data stores information about a Merkle tree.  It is initialized by `create_tree` and is updated by all other Bubblegum instructions except for decompression.
The account data is represented by the [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) struct.

| Field                              | Offset | Size | Description
| ---------------------------------- | ------ | ---- | --
| &mdash;                            | 0      | 8    | Anchor account discriminator.
| `tree_creator`                     | 8      | 32   | `PubKey` of the creator/owner of the Merkle tree.
| `tree_delegate`                    | 40     | 32   | `PubKey` of the delegate authority of the tree.  Initially it is set to the `tree_creator`.
| `num_minted`                       | 72     | 8    | `u64` that keeps track of the number of NFTs minted into the tree.  This value is very important as it is used as a nonce ("number used once") value for leaf operations to ensure the Merkle tree leaves are unique.  The nonce is basically the tree-scoped unique ID of the asset.  In practice for each asset it is retrieved from off-chain data store.

### 📄 `voucher`

The `voucher` PDA account is used when a compressed NFT is redeemed and decompressed.  It is initialized by `redeem` and represented by the [`Voucher`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L36) struct, which includes a reference to the [`LeafSchema`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/leaf_schema.rs#L45) struct.

| Field                             | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| &mdash;                           | 0      | 8    | Anchor account discriminator.
| `leaf_schema`                     | 8      | 32   | `PubKey` of the creator/owner of the Merkle tree.
| `index`                           | 40     | 32   | `PubKey` of the delegate authority of the tree.  Initially it is set to the `tree_creator`.
| `merkle_tree`                     | 72     | 32   | `PubKey` of the Merkle tree to which the leaf belonged before it was redeemed.

## Instructions

### 📄 `create_tree`

This instruction creates a Merkle Tree.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| --------------------------------- | :------: | :----: | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account that is initialized by this instruction.
| `merkle_tree`                     |    ✅    |        | The account that will contain the Merkle tree.
| `payer`                           |          |   ✅   | Payer of the transaction.
| `tree_creator`                    |          |   ✅   | The creator/owner of the Merkle tree.
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.
| `compression_program`             |          |        | The Solana Program Library spl-account-compression program ID.
| `system_program`                  |          |        | The Solana System Program ID.

</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| `max_depth`                       | 0      | 4    | The maximum depth of the Merkle tree.  The capacity of the Merkle tree is 2 ^ max_depth.
| `max_buffer_size`                 | 4      | 4    | The minimum concurrency limit of the Merkle tree.  See [Solana Program Library documentation](https://docs.rs/spl-account-compression/0.1.3/spl_account_compression/spl_account_compression/fn.init_empty_merkle_tree.html) on this for more details.

</details>

### 📄 `set_tree_delegate`

This instruction delegates authority for a previously created Merkle tree.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| --------------------------------- | :------: | :----: | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account previously initialized by `create_tree`.
| `tree_creator`                    |          |   ✅   | The creator/owner of the Merkle tree.
| `new_tree_delegate`               |          |        | The wallet to which to delegate tree authority.
| `merkle_tree`                     |    ✅    |        | The account that contains the Merkle tree, initialized by `create_tree`.

</details>

<details>
  <summary>Arguments</summary>

None.

</details>

### 📄 `mint_v1`

This instruction mints a compressed NFT.  Note that Merkle proofs are *not* required for minting.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| -----------------------------     | :------: | :----: | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account previously initialized by `create_tree`.
| `leaf_owner`                      |          |        | The wallet that will be the NFT owner.
| `leaf_delegate`                   |          |        | The wallet that will be the NFT delegate.
| `merkle_tree`                     |    ✅    |        | The account that contains the Merkle tree, initialized by `create_tree`.
| `payer`                           |          |   ✅   | Payer of the transaction.
| `tree_delegate`                   |          |   ✅   | The owner or delegate authority of the Merkle tree.
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.
| `compression_program`             |          |        | The Solana Program Library spl-account-compression program ID.

</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| `data`                            | 0      | ~    | [`MetadataArgs`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/metaplex_adapter.rs#L81) object.

</details>

### 📄 `verify_creator` and `unverify_creator`

Verify or unverify a creator that exists in the NFT's creators array.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| --------------------------------- | :------: | :----: | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account previously initialized by `create_tree`.
| `leaf_owner`                      |          |        | The NFT owner.
| `leaf_delegate`                   |          |        | The NFT delegate.
| `merkle_tree`                     |    ✅    |        | The account that contains the Merkle tree, initialized by `create_tree`.
| `payer`                           |          |   ✅   | Payer of the transaction.
| `creator`                         |          |   ✅   | The NFT creator that is signing so that the creator is set to `verified` for the NFT.
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.
| `compression_program`             |          |        | The Solana Program Library spl-account-compression program ID.
| _remaining accounts_              |          |        | `Pubkeys`(s) that are 32-byte Keccak256 hash values that represent the nodes for this NFT's Merkle proof.


</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| ----------------------------------| ------ | ---- | --
| `root`                            | 0      | 32   | The Merkle root for the tree.  Can be retrieved from off-chain data store.
| `data_hash`                       | 32     | 32   | The Keccak256 hash of the NFTs existing metadata (**without** the `verified` flag for the creator changed).  The metadata is retrieved from off-chain data store.
| `creator_hash`                    | 64     | 32   | The Keccak256 hash of the NFTs existing creators array (**without** the `verified` flag for the creator changed).  The creators array is retrieved from off-chain data store.
| `nonce`                           | 96     | 8    | A nonce ("number used once") value used to make the Merkle tree leaves unique.  This is the value of `num_minted` for the tree stored in the [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) account at the time the NFT was minted.  The unique value for each asset can be retrieved from off-chain data store.
| `index`                           | 104    | 4    | The index of the leaf node in the Merkle tree.  Can be retrieved from off-chain data store.
| `data`                            | 108    | ~    | [`MetadataArgs`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/metaplex_adapter.rs#L81) object (**without** the `verified` flag for the creator changed).  Can be retrieved from off-chain data store.

</details>

### 📄 `verify_collection`, `unverify_collection`, and `set_and_verify_collection`

Verify or unverify an NFT as a member of a Metaplex [`Certified Collection`](https://docs.metaplex.com/programs/token-metadata/certified-collections) when the collection is already set in the Metadata.  Or set a new collection in the metadata and verify the NFT as a member of the new collection.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| ----------------------------------| :------: | :----: | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account previously initialized by `create_tree`.
| `leaf_owner`                      |          |        | The NFT owner.
| `leaf_delegate`                   |          |        | The NFT delegate.
| `merkle_tree`                     |    ✅    |        | The account that contains the Merkle tree, initialized by `create_tree`.
| `payer`                           |          |   ✅   | Payer of the transaction.
| `tree_delegate`                   |          |  ❓✅  | The owner or delegate authority of the Merkle tree.  This account is checked to be a signer in the case of `set_and_verify_collection` where we are actually changing the NFT metadata.
| `collection_authority`            |          |   ✅   | Either the true collection authority a delegated collection authority (if delegated then a Collection Authority Record PDA must be provided).
| `collection_authority_record_pda` |          |        | In the case of a delegated collection authority, this is the collection authority record PDA.  See the Metaplex documentation on [`Certified Collections`](https://docs.metaplex.com/programs/token-metadata/certified-collections) for more information on verifying collections.  If there is no collecton authority record PDA then this must be the Bubblegum program address.
| `collection_mint`                 |          |        | Mint account of the collection.
| `collection_metadata`             |   ❓✅   |        | Metadata account of the collection.  Modified in the case of a sized collection.
| `edition_account`                 |          |        | Master Edition account of the collection.
| `bubblegum_signer`                |          |        | Signing PDA used when doing a CPI into token-metadata to update the collection information.
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.
| `compression_program`             |          |        | The Solana Program Library spl-account-compression program ID.
| `token_metadata_program`          |          |        | Metaplex `TokenMetadata` program ID.
| _remaining accounts_              |          |        | `Pubkeys`(s) that are 32-byte Keccak256 hash values that represent the nodes for this NFT's Merkle proof.

</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| `root`                            | 0      | 32   | The Merkle root for the tree.  Can be retrieved from off-chain data store.
| `data_hash`                       | 32     | 32   | The Keccak256 hash of the NFTs existing metadata (**without** the `verified` flag for the creator changed).  The metadata is retrieved from off-chain data store.
| `creator_hash`                    | 64     | 32   | The Keccak256 hash of the NFTs existing creators array (**without** the `verified` flag for the creator changed).  The creators array is retrieved from off-chain data store.
| `nonce`                           | 96     | 8    | A nonce ("number used once") value used to make the Merkle tree leaves unique.  This is the value of `num_minted` for the tree stored in the [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) account at the time the NFT was minted.  The unique value for each asset can be retrieved from off-chain data store.
| `index`                           | 104    | 4    | The index of the leaf node in the Merkle tree.  Can be retrieved from off-chain data store.
| `data`                            | 108    | ~    | [`MetadataArgs`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/metaplex_adapter.rs#L81) object (**without** the `verified` flag for the collection changed).  Can be retrieved from off-chain data store.
| `collection`                      | ~      | 32   | Mint address of a new Collection NFT.

</details>

### 📄 `transfer`

Transfer an NFT to a different owner.  When NFTs are transferred there is no longer a delegate authority.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| --------------------------------- | -------- | ------ | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account previously initialized by `create_tree`.
| `leaf_owner`                      |          |  ❓✅  | The NFT owner.  Transfers must be signed by either the NFT owner or NFT delegate.
| `leaf_delegate`                   |          |  ❓✅  | The NFT delegate.  Transfers must be signed by either the NFT owner or NFT delegate.
| `new_leaf_owner`                  |          |        | The wallet that will be the new NFT owner.
| `merkle_tree`                     |    ✅    |        | The account that contains the Merkle tree, initialized by `create_tree`.
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.
| `compression_program`             |          |        | The Solana Program Library spl-account-compression program ID.
| _remaining accounts_              |          |        | `Pubkeys`(s) that are 32-byte Keccak256 hash values that represent the nodes for this NFT's Merkle proof.

</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| `root`                            | 0      | 32   | The Merkle root for the tree.  Can be retrieved from off-chain data store.
| `data_hash`                       | 32     | 32   | The Keccak256 hash of the NFTs existing metadata (**without** the `verified` flag for the creator changed).  The metadata is retrieved from off-chain data store.
| `creator_hash`                    | 64     | 32   | The Keccak256 hash of the NFTs existing creators array (**without** the `verified` flag for the creator changed).  The creators array is retrieved from off-chain data store.
| `nonce`                           | 96     | 8    | A nonce ("number used once") value used to make the Merkle tree leaves unique.  This is the value of `num_minted` for the tree stored in the [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) account at the time the NFT was minted.  The unique value for each asset can be retrieved from off-chain data store.
| `index`                           | 104    | 4    | The index of the leaf node in the Merkle tree.  Can be retrieved from off-chain data store.

</details>

### 📄 `delegate`

Delegate authority of an NFT to a different wallet.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| --------------------------------- | -------- | ------ | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account previously initialized by `create_tree`.
| `leaf_owner`                      |          |   ✅   | The NFT owner.
| `previous_leaf_delegate`          |          |        | The previous NFT delegate.
| `new_leaf_delegate`               |          |        | The wallet that will be the new NFT delegate.
| `merkle_tree`                     |    ✅    |        | The account that contains the Merkle tree, initialized by `create_tree`.
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.
| `compression_program`             |          |        | The Solana Program Library spl-account-compression program ID.
| _remaining accounts_              |          |        | `Pubkeys`(s) that are 32-byte Keccak256 hash values that represent the nodes for this NFT's Merkle proof.

</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| `root`                            | 0      | 32   | The Merkle root for the tree.  Can be retrieved from off-chain data store.
| `data_hash`                       | 32     | 32   | The Keccak256 hash of the NFTs existing metadata (**without** the `verified` flag for the creator changed).  The metadata is retrieved from off-chain data store.
| `creator_hash`                    | 64     | 32   | The Keccak256 hash of the NFTs existing creators array (**without** the `verified` flag for the creator changed).  The creators array is retrieved from off-chain data store.
| `nonce`                           | 96     | 8    | A nonce ("number used once") value used to make the Merkle tree leaves unique.  This is the value of `num_minted` for the tree stored in the [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) account at the time the NFT was minted.  The unique value for each asset can be retrieved from off-chain data store.
| `index`                           | 104    | 4    | The index of the leaf node in the Merkle tree.  Can be retrieved from off-chain data store.

</details>

### 📄 `burn`

Burn an NFT.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| --------------------------------- | -------- | ------ | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account previously initialized by `create_tree`.
| `leaf_owner`                      |          |  ❓✅  | The NFT owner.  Burn must be signed by either the NFT owner or NFT delegate.
| `leaf_delegate`                   |          |  ❓✅  | The NFT delegate.  Burn must be signed by either the NFT owner or NFT delegate.
| `merkle_tree`                     |    ✅    |        | The account that contains the Merkle tree, initialized by `create_tree`.
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.
| `compression_program`             |          |        | The Solana Program Library spl-account-compression program ID.
| _remaining accounts_              |          |        | `Pubkeys`(s) that are 32-byte Keccak256 hash values that represent the nodes for this NFT's Merkle proof.

</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| `root`                            | 0      | 32   | The Merkle root for the tree.  Can be retrieved from off-chain data store.
| `data_hash`                       | 32     | 32   | The Keccak256 hash of the NFTs existing metadata (**without** the `verified` flag for the creator changed).  The metadata is retrieved from off-chain data store.
| `creator_hash`                    | 64     | 32   | The Keccak256 hash of the NFTs existing creators array (**without** the `verified` flag for the creator changed).  The creators array is retrieved from off-chain data store.
| `nonce`                           | 96     | 8    | A nonce ("number used once") value used to make the Merkle tree leaves unique.  This is the value of `num_minted` for the tree stored in the [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) account at the time the NFT was minted.  The unique value for each asset can be retrieved from off-chain data store.
| `index`                           | 104    | 4    | The index of the leaf node in the Merkle tree.  Can be retrieved from off-chain data store.

</details>

### 📄 `redeem`

Redeem an NFT (remove from tree and store in a voucher PDA).

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| --------------------------------- | -------- | ------ | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account previously initialized by `create_tree`.
| `leaf_owner`                      |          |  ✅    | The NFT owner.
| `leaf_delegate`                   |          |        | The NFT delegate.
| `merkle_tree`                     |    ✅    |        | The account that contains the Merkle tree, initialized by `create_tree`.
| `voucher`                         |    ✅    |        | [`Voucher`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L36) PDA account that is initialized by this instruction.
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.
| `compression_program`             |          |        | The Solana Program Library spl-account-compression program ID.
| `system_program`                  |          |        | The Solana System Program ID.
| _remaining accounts_              |          |        | `Pubkeys`(s) that are 32-byte Keccak256 hash values that represent the nodes for this NFT's Merkle proof.

</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| `root`                            | 0      | 32   | The Merkle root for the tree.  Can be retrieved from off-chain data store.
| `data_hash`                       | 32     | 32   | The Keccak256 hash of the NFTs existing metadata (**without** the `verified` flag for the creator changed).  The metadata is retrieved from off-chain data store.
| `creator_hash`                    | 64     | 32   | The Keccak256 hash of the NFTs existing creators array (**without** the `verified` flag for the creator changed).  The creators array is retrieved from off-chain data store.
| `nonce`                           | 96     | 8    | A nonce ("number used once") value used to make the Merkle tree leaves unique.  This is the value of `num_minted` for the tree stored in the [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) account at the time the NFT was minted.  The unique value for each asset can be retrieved from off-chain data store.
| `index`                           | 104    | 4    | The index of the leaf node in the Merkle tree.  Can be retrieved from off-chain data store.

</details>

### 📄 `cancel_redeem`

Cancel the redemption of an NFT (Put the NFT back into the Merkle tree).

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| --------------------------------- | -------- | ------ | --
| `tree_authority`                  |    ✅    |        | The [`TreeConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L17) PDA account previously initialized by `create_tree`.
| `leaf_owner`                      |          |  ✅    | The NFT owner.
| `merkle_tree`                     |    ✅    |        | The account that contains the Merkle tree, initialized by `create_tree`.
| `voucher`                         |    ✅    |        | [`Voucher`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L36) PDA account previously initialized by `redeem`.
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.
| `compression_program`             |          |        | The Solana Program Library spl-account-compression program ID.
| _remaining accounts_              |          |        | `Pubkeys`(s) that are 32-byte Keccak256 hash values that represent the nodes for this NFT's Merkle proof.

</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| `root`                            | 0      | 32   | The Merkle root for the tree.  Can be retrieved from off-chain data store.

</details>

### 📄 `decompress_v1`

Decompress an NFT into an uncompressed Metaplex NFT.  This will cost rent for the token-metadata Metadata and Master Edition accounts that are created.  Note that Merkle proofs are *not* required for decompression because the leaf (NFT) was already removed from the tree.

<details>
  <summary>Accounts</summary>

| Name                              | Writable | Signer | Description
| ----------------------------------| :------: | :----: | --
| `voucher`                         |    ✅    |        | [`Voucher`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/mod.rs#L36) PDA account previously initialized by `redeem`.
| `leaf_owner`                      |          |   ✅   | The NFT owner.
| `token_account`                   |    ✅    |        | Token account for the NFT.  This is created if it doesn't exist.
| `mint`                            |    ✅    |        | Mint PDA account for the NFT.  This is created if it doesn't exist.
| `mint_authority`                  |          |        | PDA account for mint authority.
| `metadata`                        |    ✅    |        | New token-metadata Metadata account for the NFT.  Initialized in Token Metadata Program.
| `master_edition`                  |    ✅    |        | New Master Edition account for the NFT.  Initialized in Token Metadata Program
| `system_program`                  |          |        | The Solana System Program ID.
| `sysvar_rent`                     |          |        | `Rent` account.
| `token_metadata_program`          |          |        | Metaplex `TokenMetadata` program ID.
| `token_program`                   |          |        | Solana Program Library spl-token program ID. 
| `associated_token_program`        |          |        | Solana Program Library spl-associated-token-account program ID. 
| `log_wrapper`                     |          |        | The Solana Program Library Wrapper (spl-noop) program ID.

</details>

<details>
  <summary>Arguments</summary>

| Argument                          | Offset | Size | Description
| --------------------------------- | ------ | ---- | --
| `data`                            | 0      | ~    | [`MetadataArgs`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/bubblegum/program/src/state/metaplex_adapter.rs#L81) object.

</details>

