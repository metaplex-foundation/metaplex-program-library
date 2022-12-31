# Programmable NFT Guide

## Technical Summary

In order to support assets that can have customizable behavior, a new asset class will be introduced into Token Metadata’s `Token Standard` struct. This new token standard will allow for flexible configuration of various lifecycle rules, which will be triggered at specific actions:

- Burn
- Delegate
- Mint
- Revoke
- Transfer
- Update
- Use

These lifecycle rules will be configured by creators – e.g., creators may choose to include rules for transfer restrictions (e.g., for royalties enforcement) or only allow updates with an additional signer present in the transaction.

Interaction with assets will be provided by Token Metadata:

1. Transfer instructions (and other spl-token instructions) are now sent to Token Metadata instead.
2. Token Metadata will expose new versioned instructions under an unified and simplified API. Spl-token proxy instructions are close to the existing instruction interface with the addition of a new required `authorization_rules` account argument. E.g., `CreateMetadataAccount` and `UpdateMetadata` are replaced with `mint` and `update`.
3. The `authorization_rules` account can be easily discovered on-chain using account derivation or via the Metaplex Read API, an RPC indexing extension run by many existing RPC providers.

## Extending the `TokenStandard`

A new asset class on the Token Metadata will be added to the `TokenStandard` struct:

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

To interact with the new asset class, a new set of instructions will be added to Token Metadata. It is important to note that current instructions will continue to work using the existing token standards – the new instructions will be required for interacting with `ProgrammableNonFungible` assets. At the same time, the **new instructions will support all asset classes** so all interaction can happen via an unified interface regardless of the asset class.

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
    // Authorizes the use of a token
    UseAsset(UseAssetArgs),
    // Verifies creator/collection for an asset
    Verify(VerifyArgs),
}
```

Each of these instructions will use versioned `*Args` structs to facilitate future updates, and in turn, not require additional instructions.

## Instruction Builders (Rust)

Each instruction will include an instruction builder to facilitate its creation.

<aside>
🚧 The instruction builders examples below are a draft specification.
</aside>

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
        amount,
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
        authorization_data: None,
        amount: 1,
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

### Example Rules

```rust
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Rule {
    All { rules: Vec<Rule> },
    Any { rules: Vec<Rule> },
    AdditionalSigner { account: Pubkey },
    PubkeyMatch { destination: Pubkey },
    DerivedKeyMatch { account: Pubkey },
    ProgramOwned { program: Pubkey },
    Amount { amount: u64 },
    Frequency { freq_account: Pubkey },
    PubkeyTreeMatch { root: [u8; 32] },
}
```

### RuleSets

**RuleSets** are creator or Metaplex-managed sets of rules to enforce specific behaviors for specific operations.

### Example RuleSet

```rust
// Rule for Transfers: Allow transfers to a Token Owned Escrow account.
let program_is_token_metadata = Rule::ProgramOwned {
    program: mpl_token_metadata::id(),
};

// Rule for Delegate and SaleTransfer: The provided leaf node must be a
// member of the marketplace Merkle tree.
let leaf_in_marketplace_tree = Rule::PubkeyTreeMatch {
    root: marketplace_tree_root,
};

// Create Basic Royalty Enforcement Ruleset.
let mut basic_royalty_enforcement_rule_set = RuleSet::new();
basic_royalty_enforcement_rule_set.add(Operation::Transfer, program_is_token_metadata);
basic_royalty_enforcement_rule_set.add(Operation::Delegate, leaf_in_marketplace_tree);
basic_royalty_enforcement_rule_set.add(Operation::SaleTransfer, leaf_in_marketplace_tree);
```

### RuleSet MessagePack Pre-serialization

Due to the recursive nature of RuleSets, they are incompatible with Borsh serialization and thus must be pre-serialized into the MessagePack format on the client side before sending to the `create` instruction.  Thus, the usual way to specify a RuleSet will be in JSON, and a CLI will be provided to do the serialization.

```json
{
    "Transfer": {
        "ProgramOwned": {
            "program": [11, 112, 101, 177, 227, 209, 124, 69, 56, 157, 82, 127, 107, 4, 195, 205, 88, 184, 108, 115, 26, 160, 253, 181, 73, 182, 209, 188, 3, 248, 41, 70]
        }
    },
    "Delegate": {
        "PubkeyTreeMatch": {
            "root": [42, 157, 245, 156, 21, 37, 147, 96, 183, 190, 206, 14, 24, 1, 106, 49, 167, 236, 38, 73, 98, 53, 60, 9, 154, 164, 240, 126, 210, 197, 76, 235]
        }
    },
    "SaleTransfer": {
        "PubkeyTreeMatch": {
            "root": [42, 157, 245, 156, 21, 37, 147, 96, 183, 190, 206, 14, 24, 1, 106, 49, 167, 236, 38, 73, 98, 53, 60, 9, 154, 164, 240, 126, 210, 197, 76, 235]
        }
    }   
}
```
