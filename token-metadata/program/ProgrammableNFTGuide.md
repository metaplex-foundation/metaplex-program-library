# Programmable NFT Guide

> **Warning**
> This is an alpha release, currently only available on devnet. 
>
> * :crab: Rust crate: [v1.7.0-alpha.1](https://crates.io/crates/mpl-token-metadata/1.7.0-alpha.1)
> * :package: NPM package: [v2.6.0-alpha.1](https://www.npmjs.com/package/@metaplex-foundation/mpl-token-metadata/v/2.6.0-alpha.1)
>
> **Note:** The instructions are subject to changes.

## Technical Summary

In order to support assets that can have customizable behavior, a new asset class will be introduced into Token Metadata’s `Token Standard` struct. This new token standard will allow for flexible configuration of various lifecycle rules, which will be triggered at specific actions:

- :small_white_circle: `Burn`
- [:small_blue_circle: `Create`](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L518-L536) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/create.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/create.rs))</span>
- [:small_blue_circle: `Delegate`](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L562-L585) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/delegate.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/delegate.rs))</span>
- [:small_blue_circle: `Lock`](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L607-L624) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/lock.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/lock.rs))</span>
- :small_white_circle: `Migrate`
- :small_white_circle: `Print`
- [:small_blue_circle: `Mint`](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L538-L560) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/mint.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/mint.rs))</span>
- [:small_blue_circle: `Revoke`](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L587-L605) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/revoke.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/revoke.rs))</span>
- [:small_blue_circle: `Transfer`](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L661-L683) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/transfer.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/transfer.rs))</span>
- [:small_blue_circle: `Unlock`](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L626-L643) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/unlock.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/unlock.rs))</span>
- [:small_blue_circle: `Update`](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/src/instruction/mod.rs#L685-L702) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/js/test/update.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/feat/programmable-asset/token-metadata/program/tests/update.rs))</span>
- :small_white_circle: `Use`
- :small_white_circle: `Verify`

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
    // Locks a token
    Lock(LockArgs),
    // Mint a token
    Mint(MintArgs),
    // Mint copies of a fungible asset
    Print(PrintArgs),
    // Revokes a delegate
    Revoke(RevokeArgs),
    // Transfer an asset
    Transfer(TransferArgs),
    // Unlocks a token
    Unlock(LockArgs),
    // Updates the metadata of an asset
    Update(UpdateArgs),
    // Authorizes the use of a token
    Use(UseArgs),
    // Verifies creator/collection for an asset
    Verify(VerifyArgs),
}
```

Each of these instructions will use versioned `*Args` structs to facilitate future updates, and in turn, not require additional instructions. Operations supported under each instruction are as follows:

- [ ] `Burn`
- [x] `Create`
    * [x] Creation of Programmable Non-Fungibles tokens (pNFT)
    * [x] Creation of Non-Fungibles tokens (NFT)
    * [x] Creation of Fungible Assets (*semi-fungible tokens*)
    * [x] Creation of Fungible Tokens (*fungible tokens*)
- [ ] `Delegate`
    * [ ] Creation of `Authority` delegates
    * [x] Creation of `Collection` delegates
    * [x] Creation of `Sale` delegates
    * [x] Creation of `Transfer` delegates
    * [x] Creation of `Update` delegates
    * [ ] Creation of `Use` delegates
    * [x] Creation of `Utility` delegates
- [x] `Lock`
    * [x] Lock Programmable Non-Fungibles
    * [x] Lock Non-Fungibles
    * [x] Lock Fungible Assets
    * [x] Lock Fungibles
- [ ] `Migrate`
- [x] `Mint`
    * [x] Mint Programmable Non-Fungibles tokens (pNFT)
    * [x] Mint of Non-Fungibles tokens (NFT)
    * [x] Mint Fungible Assets (*semi-fungible tokens*)
    * [x] Mint Fungible Tokens (*fungible tokens*)
- [ ] `Print`
    * [ ] Print of editions
- [ ] `Revoke`
    * [ ] Revoke of `Authority` delegates
    * [x] Revoke of `Collection` delegates
    * [x] Revoke of `Sale` delegates
    * [x] Revoke of `Transfer` delegates
    * [x] Revoke of `Update` delegates
    * [ ] Revoke of `Use` delegates
    * [x] Revoke of `Utility` delegates
- [x] `Transfer`
    * [x] wallet-to-wallet transfers
    * [x] wallet-to-program transfers
    * [x] program-to-wallet transfers
    * [x] program-to-program transfers
- [x] `Update`
    * [x] Update metadata details for Programmable Non-Fungibles
    * [x] Update metadata details for Non-Fungibles
    * [x] Update metadata details for Fungibles Assets
    * [x] Update metadata details for Fungibles
- [x] `Unlock`
    * [x] Unlock Programmable Non-Fungibles
    * [x] Unlock Non-Fungibles
    * [x] Unlock Fungible Assets
    * [x] Unlock Fungibles
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

> **Warning**
> The instruction builders examples below are a draft specification.

Each instruction will include an instruction builder to facilitate its creation. Each instruction has an associated builder, which will validate that all required accounts are provided and set the default values for any of the optional accounts that are not set.

### Example: `Create` instruction builder

Creates the metadata account for a `ProgrammableNonFungible` asset, initializing the mint account if needed.

```rust
use mpl_token_metadata::instruction::builders::CreateBuilder;

...

let name = puffed_out_string("Programmable NFT", MAX_NAME_LENGTH);
let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
let uri = puffed_out_string("http://first.programmable.nft/", MAX_URI_LENGTH);

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

## JS SDK

Token Metadata includes a low-leve Solita-based SDK, which can be used to interact with the new API. The NPM package can be found [here](https://www.npmjs.com/package/@metaplex-foundation/mpl-token-metadata/v/2.6.0-alpha.1).

## Token Authorization Rules

There will be a separate Token Authorization Rules program that provides the ability to create and execute rules to restrict the token operations discussed above.

### Overview

Authorization rules are variants of a `Rule` enum that implements a `validate()` function.

There are **Primitive Rules** and **Composed Rules** that are created by combining of one or more primitive rules:

- **Primitive Rules:** store any accounts or data needed for evaluation, and at runtime will produce a `true` or `false` output based on accounts and a well-defined `Payload` that are passed into the `validate()` function.
- **Composed Rules:** return a `true` or `false` based on whether any or all of the primitive rules return `true`.  Composed rules can then be combined into higher-level composed rules that implement more complex boolean logic.  Because of the recursive definition of the `Rule` enum, calling `validate()` on a top-level composed rule will start at the top and validate at every level, down to the component primitive rules.

More details of the Token Authorization Rules program, including examples, can be found [here](https://github.com/metaplex-foundation/mpl-token-auth-rules/blob/main/README.md).
