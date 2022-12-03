# Programmable NFT Guide

## Technical Summary

In order to support assets that can have customizable behavior, a new asset class will be introduced into Token Metadata’s `Token Standard` struct. This new token standard will allow for flexible configuration of various lifecycle rules, which will be triggered at specific actions:

- Mint
- Burn
- Update
- Transfer
- Delegate
- Revoke
- Use

These lifecycle rules will be configured by creators – e.g., creators may choose to include rules for transfer restrictions (e.g., for royalties enforcement) or only allow updates with an additional signer is present in the transaction.

Interaction with assets will be provided by Token Metadata:

1. Transfer instructions (and other spl-token instructions) are now sent to Token Metadata instead.
2. Token Metadata will expose new versioned instructions under an unified and simplified API. Spl-token proxy instructions are close to the existing instruction interface with the addition of a new required `authorization_rules` account argument. E.g., `CreateMetadataAccount` and `UpdateMetadata` are replaced with `mint` and `update`.
3. The `authorization_rules` can be easily discovered on chain using account derivation or via the Metaplex Read API, an RPC indexing extension run by many existing RPC providers.

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
  /// An NFT with customizale behaviour for lifecycle events
  /// (e.g. transfers, updates, etc.).
	**ProgrammableNonFungible**,
}
```

When a `ProgrammableNonFungible` asset is created, it will have a `RuleSet` associated. These rules are then validated at each lifecycle action and the action is only performed if the validation succeeds. Since these rules are customizable, the tokens have a *programmable* behavior*.*

## Unified instructions

To interact with the new asset class, a new unified set of instruction will be added to Token Metadata. It is important to note that current instructions will continue to work using the existing token standard – they will be required for interacting with `ProgrammableNonFungible` assets. At the same time, the new instructions will support all asset classes so all interaction can happen via an unified interface regardless of the asset class.

Token Metadata instruction will be expanded to include:

```rust
pub enum MetadataInstruction {
    ..,
		// Mint a token with added metadata
    Mint(MintArgs),
		// Updates the metadata of an asset
		Update(UpdateArgs),
		// Closes the accounts of an asset
		Burn(BurnArgs),
		// Authorizes the use of a token
		UseAsset(UseAssetArgs),
		// Transfer an asset
		Transfer(TransferArgs),
		// Verifies creator/collection for an asset
		Verify(VerifyArgs),
		// Create a delegate
		Delegate(DelegateArgs),
		// Revokes a delegate
		Revoke(RevokeArgs),
		// Mint copies of a fungible asset
		Print(PrintArgs),
		// Change the asset type of an asset
		Migrate(MigrateArgs),
}
```

Each of these instruction will use versioned `*Args` structs to facilitate future updates, and in turn, not require additional instructions.

## Instruction Builders

Each instruction will include an instruction builder to facilitate its creation.

<aside>
🚧 The instruction builders examples below are a draft specification.

</aside>

### `Mint`

Creates an instruction to mint a new asset and create associated metadata accounts.

```rust
/// # Accounts:
///
///   0. `[writable]` Token account
///   1. `[writable]` Metadata account
///   2. `[]` Mint account
///   3. `[signer]` Mint authority
///   4. `[signer]` Payer
///   5. `[signer]` Update authority
///   6. `[]` System program
///   7. `[]` Instructions sysvar account
///   8. `[]` SPL Token program
///   9. `[]` SPL Associated Token Account program
///   10. `[optional]` Master edition account
///   11. `[optional]` Asset authorization rules account
pub fn mint(
    token: Pubkey,
    metadata: Pubkey,
    master_edition: Option<Pubkey>,
    mint: Pubkey,
    mint_authority: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    data: AssetData,
    initialize_mint: bool,
    update_authority_as_signer: bool,
);
```

Minting a new asset:

```rust
// asset details

let name = puffed_out_string("Programmable NFT", MAX_NAME_LENGTH);
let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
let uri = puffed_out_string("uri", MAX_URI_LENGTH);

let mut asset = AssetData::new(name.clone(), symbol.clone(), uri.clone());
asset.token_standard = Some(TokenStandard::ProgrammableNonFungible);
asset.seller_fee_basis_points = 500;
asset.programmable_config = Some(ProgrammableConfig {
    rule_set: Pubkey::from_str("Cex6GAMtCwD9E17VsEK4rQTbmcVtSdHxWcxhwdwXkuAN")?,
});

...

// mint instruction creation

let mint_ix = instruction::mint(
    /* token account    */ token,
    /* metadata account */ metadata,
    /* master edition   */ Some(master_edition),
    /* mint account     */ mint.pubkey(),
    /* mint authority   */ payer_pubkey,
    /* payer            */ payer_pubkey,
    /* update authority */ payer_pubkey,
    /* asset data       */ asset,
    /* initialize mint  */ true,
    /* authority signer */ true,
);

let tx = Transaction::new_signed_with_payer(
    &[mint_ix],
    Some(&context.payer.pubkey()),
    &[&context.payer, &mint],
    context.last_blockhash,
);
```

### `Update`

Creates an instruction to update an existing asset.

```rust
/// # Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[]` Mint account
///   2. `[]` System program
///   3. `[]` Instructions sysvar account
///   4. `[optional]` Master edition account
///   5. `[optional]` New update authority
///   6. `[signer, optional]` Update authority
///   7. `[signer, optional]` Token holder
///   8. `[optional]` Token account
///   9. `[optional]` Asset authorization rules account
///   10. `[optional]` Authorization rules program
pub fn update(
    metadata_account: Pubkey,
    mint_account: Pubkey,
    master_edition_account: Option<Pubkey>,
    new_update_authority: Option<Pubkey>,
    authority: AuthorityType,
    authorization_rules: Option<Pubkey>,
    data: Option<AssetData>,
    additional_accounts: Option<Vec<AccountMeta>>,
) -> Instruction;
```

Updating an asset requires to retrieve the asset information, make the required modification and submit the update transaction.

```rust
let metadata_account = get_account(context, &metadata_pubkey).await;
let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
metadata.name = puffed_out_string("Updated Programmable NFT", MAX_NAME_LENGTH);
metadata.seller_fee_basis_points = 500;

...

let payer_pubkey = context.payer.pubkey();
let new_update_authority = None;
let authority = AuthorityType::UpdateAuthority(payer_pubkey);
let update_ix = instruction::update(
    /* metadata account */ metadata_pubkey,
    /* mint account     */ mint.pubkey(),
    /* master edition   */ None,
    /* new auth         */ new_update_authority,
    /* authority        */ authority,
    /* auth rules       */ None,
    /* asset data       */ Some(metadata),
    /* additional       */ None,
);

...

let tx = Transaction::new_signed_with_payer(
    &[update_ix],
    Some(&context.payer.pubkey()),
    &[&context.payer],
    context.last_blockhash,
);
```

### `Transfer`

Transfer an asset. When transferring a `ProgrammableNonFungible` asset, it is required to send the `authorization_rules` account to allow the validation of the transfer.

```rust
/// # Accounts:
///   0. `[writable]` Token account
///   1. `[writable]` Metadata account
///   2. `[]` Mint account
///   3. `[]` Destination account
///   4. `[writable]` Destination associate token account
///   5. `[signer]` Owner
///   6. `[]` STL Token program
///   7. `[]` STL Associate Token program
///   8. `[]` System program
///   9. `[]` Instructions sysvar account
///   10. `[optional]` Asset authorization rules account
///   11. `[optional]` Token Authorization Rules program
pub fn transfer(
    token_account: Pubkey,
    metadata_account: Pubkey,
    mint_account: Pubkey,
    destination: Pubkey,
    destination_token_account: Pubkey,
    owner: Pubkey,
    args: TransferArgs,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
) -> Instruction;
```

### `Delegate`

Creates a delegate for a token to perform specific actions.

```rust
/// # Accounts:
///
///   0. `[writable]` Delegate account key
///   1. `[]` Delegated user
///   2. `[signer]` Token owner
///   3. `[signer, writable]` Payer
///   4. `[writable]` Owned token account of mint
///   5. `[writable]` Metadata account
///   6. `[]` Mint of metadata
///   7. `[]` System Program
///   8. `[]` SPL Token Program
///   9. `[]` Instructions sysvar account
///   10. `[optional]` Token Authorization Rules account
///   11. `[optional]` Token Authorization Rules Program
pub fn delegate(
    delegate: Pubkey,
    user: Pubkey,
    token_owner: Pubkey,
    payer: Pubkey,
    token: Pubkey,
    metadata: Pubkey,
    args: DelegateArgs,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
) -> Instruction;
```

### `Revoke`

Revokes an existing delegate.

```rust
/// # Accounts:
///
///   0. `[writable]` Delegate account key
///   1. `[]` Delegated user
///   2. `[signer]` Token owner
///   3. `[signer, writable]` Payer
///   4. `[writable]` Owned token account of mint
///   5. `[writable]` Metadata account
///   6. `[]` Mint of metadata
///   7. `[]` System Program
///   8. `[]` SPL Token Program
///   9. `[]` Instructions sysvar account
///   10. `[optional]` Token Authorization Rules account
///   11. `[optional]` Token Authorization Rules Program
pub fn revoke(
    delegate: Pubkey,
    user: Pubkey,
    token_owner: Pubkey,
    payer: Pubkey,
    token: Pubkey,
    metadata: Pubkey,
    args: RevokeArgs,
    authorization_rules: Option<Pubkey>,
    additional_accounts: Option<Vec<AccountMeta>>,
) -> Instruction;
```

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
