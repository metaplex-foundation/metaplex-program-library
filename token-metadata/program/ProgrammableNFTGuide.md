# Programmable NFT Guide

## Technical Summary

In order to support assets that can have customizable behavior, a new asset class will be introduced into Token Metadata’s `Token Standard` struct. This new token standard will allow for flexible configuration of various lifecycle rules, which will be triggered at specific actions:

- Burn
- Create
- Delegate
- Migrate
- Mint
- Print
- Revoke
- Transfer
- Update
- Utility
- Verify

These lifecycle rules will be configured by creators – e.g., creators may choose to include rules for transfer restrictions (e.g., for royalties enforcement) or only allow updates with an additional signer present in the transaction.

Interaction with assets will be provided by Token Metadata:

1. Transfer instructions (and other spl-token instructions) are now sent to Token Metadata instead.
2. Token Metadata will expose new versioned instructions under an unified and simplified API. Spl-token proxy instructions are close to the existing instruction interface with the addition of a new required `authorization_rules` account argument. E.g., `CreateMetadataAccount` and `UpdateMetadata` are replaced with `Create` and `Update`.
3. The `authorization_rules` account can be easily discovered on-chain using account derivation or via the Metaplex Read API, an RPC indexing extension run by many existing RPC providers.

## Extending the `TokenStandard`

Programmable Non-Fungibles (`pNFT`) are represented as a new asset class on the Token Metadata's `TokenStandard` struct:

```rust
pub enum TokenStandard {
    /// This is a master edition.
    NonFungible,
    /// A token with metadata that can also have attrributes.
    FungibleAsset,
    /// A token with simple metadata.
    Fungible,
    /// This is a limited edition.
    NonFungibleEdition,
    /// [NEW] An NFT with customizale behaviour for lifecycle events
    /// (e.g. transfers, updates, etc.).
    ProgrammableNonFungible,
}
```

When a `ProgrammableNonFungible` asset is created, it can have a `RuleSet` associated. These rules are then validated at each lifecycle action and the action is only performed if the validation succeeds. Since these rules are customizable, the tokens have a *programmable* behavior.

## Unified instructions

To interact with the new asset class, a new set of instructions will be added to Token Metadata. Note that current instructions will continue to work using the existing token standards – the new instructions are required for interacting with `ProgrammableNonFungible` assets. At the same time, the **new instructions will support all asset classes** so all interaction can happen via an unified interface regardless of the asset class.

Token Metadata instruction will be expanded to include:

```rust
pub enum MetadataInstruction {
    ..,
    // Closes the accounts of an asset
    Burn(BurnArgs),
    // Create the metadata and associated accounts
    Create(CreateArgs),
    // Create a delegate
    Delegate(DelegateArgs),
    // Change the asset type of an asset
    Migrate(MigrateArgs),
    // Mint a token
    Mint(MintArgs),
    // Mint copies of a fungible asset
    Print(PrintArgs),
    // Revokes a delegate
    Revoke(RevokeArgs),
    // Transfer an asset
    Transfer(TransferArgs),
    // Updates the metadata of an asset
    Update(UpdateArgs),
    // Provide utility operations on a token
    Utility(UtilityArgs),
    // Verifies creator/collection for an asset
    Verify(VerifyArgs),
}
```

Each of these instructions will use versioned `*Args` structs to facilitate future updates, and in turn, not require additional instructions. Operations supported under each instruction are as follows:

- [ ] `Burn`
- [X] `Create`
    * [X] Creation of Programmable Non-Fungibles tokens (pNFT)
    * [X] Creation of Non-Fungibles tokens (NFT)
    * [X] Creation of Fungible Assets (*semi-fungible tokens*)
    * [X] Creation of Fungible Tokens (*fungible tokens*)
- [ ] `Delegate`
    * [ ] Creation of `Authority` delegates
    * [X] Creation of `Collection` delegates
    * [X] Creation of `Sale` delegates
    * [X] Creation of `Transfer` delegates
    * [X] Creation of `Update` delegates
    * [ ] Creation of `Use` delegates
    * [X] Creation of `Utility` delegates
- [X] `Lock`
    * [X] Lock Programmable Non-Fungibles
    * [X] Lock Non-Fungibles
    * [X] Lock Fungible Assets
    * [X] Lock Fungibles
- [ ] `Migrate`
- [X] `Mint`
    * [X] Mint Programmable Non-Fungibles tokens (pNFT)
    * [X] Mint of Non-Fungibles tokens (NFT)
    * [X] Mint Fungible Assets (*semi-fungible tokens*)
    * [X] Mint Fungible Tokens (*fungible tokens*)
- [ ] `Print`
    * [ ] Print of editions
- [ ] `Revoke`
    * [ ] Revoke of `Authority` delegates
    * [X] Revoke of `Collection` delegates
    * [X] Revoke of `Sale` delegates
    * [X] Revoke of `Transfer` delegates
    * [X] Revoke of `Update` delegates
    * [ ] Revoke of `Use` delegates
    * [X] Revoke of `Utility` delegates
- [X] `Transfer`
    * [X] wallet-to-wallet transfers
    * [X] wallet-to-program transfers
    * [X] program-to-wallet transfers
- [X] `Update`
    * [X] Update metadata details for Programmable Non-Fungibles
    * [X] Update metadata details for Non-Fungibles
    * [X] Update metadata details for Fungibles Assets
    * [X] Update metadata details for Fungibles
- [X] `Unlock`
    * [X] Unlock Programmable Non-Fungibles
    * [X] Unlock Non-Fungibles
    * [X] Unlock Fungible Assets
    * [X] Unlock Fungibles
- [ ] `Verify`
    * [ ] Verify collection items
    * [ ] Verify creators

## Positional Optional Accounts

The new instruction handlers support positional optional accounts, where an account can be present or not in a transaction. When a instruction is created, it is necessary to provide a list of `PublicKeys` for the instruction accounts – e.g.:
```javascript
const mintAcccounts: MintInstructionAccounts = {
    token,
    tokenOwner,
    metadata,
    masterEdition,
    mint,
    payer,
    ...
};
```
In general, the accounts will be added to the transaction following a pre-defined position:
```javascript
// Accounts relative position:
0. token
1. tokenOwner
2. metadata
3. masterEdition
4. mint
5. payer
...
```
When you are minting from a semi-fungible token, there is no need to pass a `masterEdition` account (semi-fungibles do not have a master edition account associated). If we simply omit the `masterEdition` account, the relative position of the remaining accounts (the accounts after the master edition) would change, resulting in the program logic to be inconsistent. One way to address this is to set another `PublicKey` value to represent a "not-set-value" to maintain the position but at the same time indicate that the master edition account was not set. This is accomplished by setting the Token Metadata program key as the `PublicKey` for any account that should be ommited. This is an efficient approach since:
1. The (Token Metadata) program id is already included in the transaction by default so adding another reference to it does not take the full 32 bytes of `PublicKey` – only a single byte is used in this case;
2. The relative position of accounts is maintained since there is a public key value for the account;
3. The program can easily determine if the account key represent a "valid" public key or a "not-set-value".

Using this approach, the same handler support positional optinal account by just ommiting the `masterEdition`:
```javascript
const mintAcccounts: MintInstructionAccounts = {
    token,
    tokenOwner,
    metadata,
    mint,
    payer,
    ...
};
```
Under the hood, you the Token Metadata's `PROGRAM_ID` is set as the master edition account `PublicKey`. This will inform the program that the master edition account was not set and still maintain the relative position of the remaining accounts. Token Metadata includes a rust crate wand an NPM package with instruction builders that support positional optional accounts – you only need to set the "required" accounts using these builders.

> **Note**
> This is a similar approach used by Anchor v0.26 to support positional optional accounts

## Instruction Builders (Rust)

Each instruction will include an instruction builder to facilitate its creation.

> **Warning**
> The instruction builders examples below are a draft specification.

### `Create` ([instruction](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L506-L518) | [test source code](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/create.rs))

Creates the metadata account for a `ProgrammableNonFungible` asset, initializing the mint account if needed.

```rust
use mpl_token_metadata::instruction::builders::CreateBuilder;

...

let name = puffed_out_string("Programmable NFT", MAX_NAME_LENGTH);
let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
let uri = puffed_out_string("uri", MAX_URI_LENGTH);

let mut asset = AssetData::new(
    TokenStandard::ProgrammableNonFungible,
    name.clone(),
    symbol.clone(),
    uri.clone(),
    context.payer.pubkey(),
);
asset.seller_fee_basis_points = 500;
asset.programmable_config = Some(ProgrammableConfig {
    rule_set: Pubkey::from_str("Cex6GAMtCwD9E17VsEK4rQTbmcVtSdHxWcxhwdwXkuAN").unwrap(),
});

...

let create_ix = CreateBuilder::new()
    .metadata(metadata)
    .master_edition(master_edition)
    .mint(mint)
    .mint_authority(payer_pubkey)
    .payer(payer_pubkey)
    .update_authority(payer_pubkey)
    .initialize_mint(true)
    .update_authority_as_signer(true)
    .build(CreateArgs::V1 {
        asset_data: asset,
        decimals: Some(0),
        max_supply: Some(0),
    })?
    .instruction();
```

### `Mint` ([instruction](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L526-L539) | [test source code](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/mint.rs))

Creates an instruction to mint a new asset and create associated metadata accounts.

```rust
use mpl_token_metadata::instruction::builders::MintBuilder;

...

let mint_ix = MintBuilder::new();
    .token(token)
    .metadata(metadata)
    .master_edition(master_edition)
    .mint(mint)
    .authority(payer_pubkey)
    .payer(payer_pubkey)
    .authorization_rules(authorization_rules)
    .build(MintArgs::V1 {
        amount: 1,
        authorization_data,
    })?
    .instruction();
```

### `Update` ([instruction](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L545-L556) | [test source code](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/update.rs))

Creates an instruction to update an existing asset.

```rust
use mpl_token_metadata::instruction::builders::UpdateBuilder;

...

let mut update_args = UpdateArgs::default();
update_args.seller_fee_basis_points = Some(500);

let update_ix = UpdateBuilder::new();
    .authority(update_authority)
    .metadata(metadata)
    .mint(mint),
    .edition(master_edition),
    .build(update_args)?
    .instruction();
```

### `Transfer` ([instruction](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L598-L614) | [test source code](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/transfer.rs))

Transfer an asset. When transferring a `ProgrammableNonFungible` asset, it is required to send the `authorization_rules` account to allow the validation of the transfer.

```rust
use mpl_token_metadata::instruction::builders::TransferBuilder;

...

let mut transfer_ix = TransferBuilder::new();
    .authority(authority)
    .token_owner(source_owner)
    .token(token)
    .destination_owner(destination_owner)
    .destination(destination_token)
    .metadata(metadata)
    .mint(mint);
    .edition(master_edition);
    .authorization_rules(authorization_rules);
    .build(TransferArgs::V1 {
        amount: 1,
        authorization_data,
    })?
    .instruction();
```

### `Delegate` ([instruction](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L634-L648) | [test source code](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/delegate.rs))

Creates a delegate for a token to perform specific actions.

```rust
use mpl_token_metadata::instruction::builders::DelegateBuilder;

...

// creates a transfer delegate
let delegate_ix = DelegateBuilder::new();
    .delegate(delegate)
    .delegate_record(delegate_record)
    .mint(mint)
    .metadata(metadata)
    .payer(payer_pubkey)
    .authority(payer_pubkey)
    .master_edition(master_edition)
    .token(token)
    .build(DelegateArgs::TransferV1 {
        amount: 1,
        authorization_data,
    })?
    .instruction();
```

### `Revoke` ([instruction](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L651-L665) | [test source code](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/revoke.rs))

Revokes an existing delegate.

```rust
use mpl_token_metadata::instruction::builders::RevokeBuilder;

...

// revokes a transfer delegate
let revoke_ix = RevokeBuilder::new();
    .delegate(delegate)
    .delegate_record(delegate_record)
    .mint(mint)
    .metadata(metadata)
    .payer(payer_pubkey)
    .authority(payer_pubkey);
    .master_edition(master_edition);
    .token(token);
    .build(RevokeArgs::TransferV1)?
    .instruction();
```

## JS SDK

Token Metadata includes a low-leve Solita-based SDK, which can be used to interact with the new API. Below are a list of test examples to perform the operations:

* [Create](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/create.test.ts)
* [Delegate](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/delegate.test.ts)
* [Mint](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/mint.test.ts)
* [Revoke](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/revoke.test.ts)
* [Transfer](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/transfer.test.ts)
* [Update](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/update.test.ts)

## Token Authorization Rules

There will be a separate Token Authorization Rules program that provides the ability to create and execute rules to restrict the token operations discussed above.

### Overview

Authorization rules are variants of a `Rule` enum that implements a `validate()` function.

There are **Primitive Rules** and **Composed Rules** that are created by combining of one or more primitive rules:

- **Primitive Rules:** store any accounts or data needed for evaluation, and at runtime will produce a `true` or `false` output based on accounts and a well-defined `Payload` that are passed into the `validate()` function.
- **Composed Rules:** return a `true` or `false` based on whether any or all of the primitive rules return `true`.  Composed rules can then be combined into higher-level composed rules that implement more complex boolean logic.  Because of the recursive definition of the `Rule` enum, calling `validate()` on a top-level composed rule will start at the top and validate at every level, down to the component primitive rules.

More details of the Token Authorization Rules program, including examples, can be found [here](https://github.com/metaplex-foundation/mpl-token-auth-rules/blob/main/README.md).
