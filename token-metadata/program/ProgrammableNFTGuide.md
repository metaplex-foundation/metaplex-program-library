# Programmable NFT Guide

## Developer packages
Token Metadata
* :crab: Rust crate: [v1.11.1](https://crates.io/crates/mpl-token-metadata/1.11.1)
* :package: NPM package: [v2.11.1](https://www.npmjs.com/package/@metaplex-foundation/mpl-token-metadata/v/2.11.1)

Token Authorization Rules
* :crab: Rust crate: [v1.2.0](https://crates.io/crates/mpl-token-auth-rules/1.2.0)
* :package: NPM package: [v1.2.0](https://www.npmjs.com/package/@metaplex-foundation/mpl-token-auth-rules/v/1.2.0)

## 📄  Technical Summary

In order to support assets that can have customizable behavior, a new asset class will be introduced into Token Metadata’s `Token Standard` struct. This new token standard will allow for flexible configuration of various lifecycle rules, which will be triggered at specific actions:

* [x] [`Burn`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L504-L545)<span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/burn.test.ts)
|
[Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/burn.rs))</span>
* [x] [`Create`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L547-L565) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/create.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/create.rs))</span>
* [x] [`Delegate`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L591-L614) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/delegate.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/delegate.rs))</span>
* [x] [`Lock`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L636-L654) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/lock.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/lock.rs))</span>
* [x] [`Migrate`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L676-L693)
* [ ] `Print`
* [x] [`Mint`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L567-L589) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/mint.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/mint.rs))</span>
* [x] [`Revoke`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L616-L634) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/revoke.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/revoke.rs))</span>
* [x] [`Transfer`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L695-L717) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/transfer.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/transfer.rs))</span>
* [x] [`Unlock`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L656-L674) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/unlock.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/unlock.rs))</span>
* [x] [`Update`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L719-L735) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/update.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/update.rs))</span>
* [x] [`Unverify`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L773-L785) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/verification.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/unverify.rs))</span>
* [ ] `Use`
* [x] [`Verify`](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/instruction/mod.rs#L758-L771) <span style="font-family:'Lucida Console', monospace; font-size: 6pt">([TypeScript](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/js/test/verification.test.ts) | [Rust](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/tests/verify.rs))</span>

These lifecycle rules will be configured by creators – e.g., creators may choose to include rules for transfer restrictions (e.g., for royalties enforcement) or only allow updates with an additional signer present in the transaction.

Interaction with assets will be provided by Token Metadata:

1. Transfer instructions (and other spl-token instructions) are now sent to Token Metadata instead.
2. Token Metadata will expose new versioned instructions under a unified and simplified API. Spl-token proxy instructions are close to the existing instruction interface with the addition of a new required `authorization_rules` account argument. E.g., `CreateMetadataAccount` and `UpdateMetadata` are replaced with `Create` and `Update`.
3. The `authorization_rules` account can be easily discovered on-chain using account derivation or via the Metaplex Read API, an RPC indexing extension run by many existing RPC providers.

## 🚛  Extending the `TokenStandard`

Programmable Non-Fungibles (`pNFT`) are represented as a new asset class on the Token Metadata's `TokenStandard` struct:

```rust
pub enum TokenStandard {
    /// This is a master edition.
    NonFungible,
    /// A token with metadata that can also have attributes.
    FungibleAsset,
    /// A token with simple metadata.
    Fungible,
    /// This is a limited edition.
    NonFungibleEdition,
    /// [NEW] An NFT with customizable behaviour for lifecycle events
    /// (e.g. transfers, updates, etc.).
    ProgrammableNonFungible,
}
```

When a `ProgrammableNonFungible` asset is created, it can have a `RuleSet` associated. These rules are then validated at each lifecycle action and the action is only performed if the validation succeeds. Since these rules are customizable, the tokens have a *programmable* behavior.

The diagram below highlights the new components of a `ProgrammableNonFungible`:

![image](https://user-images.githubusercontent.com/729235/215168226-c2358a16-eab5-4b6e-af24-4ac01f21303d.png)

## ⛩️  Unified instructions

To interact with the new asset class, a new set of instructions will be added to Token Metadata. Note that current instructions will continue to work using the existing token standards – the new instructions are required for interacting with `ProgrammableNonFungible` assets. At the same time, the **new instructions will support all asset classes** so all interaction can happen via a unified interface regardless of the asset class.

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
    Verify(VerificationArgs),
}
```

Each of these instructions will use versioned `*Args` structs to facilitate future updates, and in turn, not require additional instructions. Operations supported under each instruction are as follows:

- `Burn`
    * [x] Burn Programmable Non-Fungibles
    * [x] Burn Non-Fungibles
    * [x] Burn Non-Fungible Editions
    * [x] Burn Fungibles

- `Create`
    * [x] Creation of Programmable Non-Fungible tokens (pNFT)
    * [x] Creation of Non-Fungible tokens (NFT)
    * [x] Creation of Fungible Assets (*semi-fungible tokens*)
    * [x] Creation of Fungible Tokens (*fungible tokens*)

- `Delegate`
    See Delegates Section below for more information on what the difference is between types of delegates.
    * [ ] Creation of `Authority` delegates
    * [x] Creation of `Collection` delegates
    * [x] Creation of `Sale` delegates
    * [x] Creation of `Transfer` delegates
    * [x] Creation of `Update` delegates
    * [ ] Creation of `Use` delegates
    * [x] Creation of `Utility` delegates

- `Lock`
    * [x] Lock Programmable Non-Fungibles
    * [x] Lock Non-Fungibles
    * [x] Lock Fungible Assets
    * [x] Lock Fungibles

- `Migrate`
    * [x] Migrate Non-Fungibles to Programmable Non-Fungibles

- `Mint`
    * [x] Mint Programmable Non-Fungible tokens (pNFT)
    * [x] Mint of Non-Fungible tokens (NFT)
    * [x] Mint Fungible Assets (*semi-fungible tokens*)
    * [x] Mint Fungible Tokens (*fungible tokens*)

- `Print`
    * [ ] Print of editions

- `Revoke`
    * [ ] Revoke of `Authority` delegates
    * [x] Revoke of `Collection` delegates
    * [x] Revoke of `Sale` delegates
    * [x] Revoke of `Transfer` delegates
    * [x] Revoke of `Update` delegates
    * [ ] Revoke of `Use` delegates
    * [x] Revoke of `Utility` delegates

- `Transfer`
    * [x] wallet-to-wallet transfers
    * [x] wallet-to-program transfers
    * [x] program-to-wallet transfers
    * [x] program-to-program transfers

- `Update`
    * [x] Update metadata details for Programmable Non-Fungibles
    * [x] Update metadata details for Non-Fungibles
    * [x] Update metadata details for Fungibles Assets
    * [x] Update metadata details for Fungibles

- `Unlock`
    * [x] Unlock Programmable Non-Fungibles
    * [x] Unlock Non-Fungibles
    * [x] Unlock Fungible Assets
    * [x] Unlock Fungibles

- `Unverify`
    * [x] Unverify collection items
    * [x] Unverify creators

- `Use`

- `Verify`
    * [x] Verify collection items
    * [x] Verify creators

## 🏗️  Positional Optional Accounts

The new instruction handlers support positional optional accounts, where an account can be present or not in a transaction. When an instruction is created, it is necessary to provide a list of `PublicKeys` for the instruction accounts – e.g.:
```javascript
const mintAccounts: MintInstructionAccounts = {
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
When you are minting from a semi-fungible token, there is no need to pass a `masterEdition` account (semi-fungibles do not have a master edition account associated). If we simply omit the `masterEdition` account, the relative position of the remaining accounts (the accounts after the master edition) would change, resulting in the program logic to be inconsistent. One way to address this is to set another `PublicKey` value to represent a "not-set-value" to maintain the position but at the same time indicate that the master edition account was not set. This is accomplished by setting the Token Metadata program key as the `PublicKey` for any account that should be omitted. This is an efficient approach since:
1. The (Token Metadata) program id is already included in the transaction by default so adding another reference to it does not take the full 32 bytes of `PublicKey` – only a single byte is used in this case;
2. The relative position of accounts is maintained since there is a public key value for the account;
3. The program can easily determine if the account key represents a "valid" public key or a "not-set-value".

Using this approach, the same handler supports a positional optional account by just omitting the `masterEdition`:
```javascript
const mintAccounts: MintInstructionAccounts = {
    token,
    tokenOwner,
    metadata,
    mint,
    payer,
    ...
};
```
Under the hood, the Token Metadata's `PROGRAM_ID` is set as the master edition account `PublicKey`. This will inform the program that the `masterEdition` account was not set and still maintain the relative position of the remaining accounts. Token Metadata includes a Rust crate and an NPM package with instruction builders that support positional optional accounts – you only need to set the "required" accounts using these builders.

> **Note**
> This is a similar approach used by Anchor v0.26 to support positional optional accounts

## 🧱  Instruction Builders (Rust)

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

## 👤  Delegates

The new unified API of token metadata exposes a system of delegations where other actors can be 'delegated' powers to do specific actions on the assets or asset grouping (collection).

> **Note:**
> For programmable NFTs, auth rules manage which actors can become any of these types of delegates.

### Delegate Types

There are two types of delegates on Token Metadata: `TokenDelegate` and `MetadataDelegate`. 

#### Token Delegate

`TokenDelegate`s are delegates that operate at the token level – i.e., they are spl-token delegates. This allows the delegate to perform operations on the token account (burn, transfer, freeze). There can only be one token delegate at a time and they do not have an individual delegate account associated – their information is stored on the `TokenRecord` account. The token record holds information about a particular token account (PDA seeds `["metadata", program id, mint id, "token_record", token account id]`):
```rust
pub struct TokenRecord {
    pub key: Key,
    pub bump: u8,
    pub state: TokenState,
    pub rule_set_revision: Option<u64>,
    pub delegate: Option<Pubkey>,
    pub delegate_role: Option<TokenDelegateRole>,
    pub locked_transfer: Option<Pubkey>,
}
```

`TokenState` has three different values and instructions are restricted depending on the token state value:

| **Token State** | 🔓 `Unlocked` | 🔐 `Locked` | 🏠 `Listed` |
| ------------------ | --- | --- | --- |
| *Owner Transfer*     | ✅ | ❌ | ❌ |
| *Delegate Transfer*  | 🟠 only: Sale, Transfer | ❌ | 🟠 only: Sale |
| *Owner Burn*         | ✅ | ❌ | ❌ |
| *Delegate Burn*      | 🟠 only: Utility | ❌ | ❌ |
| *Owner Revoke*       | ✅ | ❌ | ✅ → 🔓 `Unlocked` |
| *Owner Approve*      | ✅ if Sale → 🏠 `Listed` | ❌ | ❌ |
| *Owner Unlock*       | ❌ | ❌ | ❌ |
| *Delegate Unlock*    | ❌ | ✅ → 🔓 `Unlocked` | ❌ |
| *Owner Lock*         | ❌ | ❌ | ❌ |
| *Delegate Lock*      | ✅ if Utility or Staking → 🔐 `Locked` | ❌ | ❌ |
| *Mint (destination)* | ✅ | ❌ | ✅ |

`TokenDelegateRole` represents the different delegate types. There are six different values and instructions are restricted depending on the token delegate role and token state values:

| **Delegate** | None | `Sale` | `Transfer` | `LockedTransfer` | `Utility` | `Staking` | `Migration` | `Standard` (SPL)  |
| --------------------- | --- | --- | --- | --- | --- | --- | --- | --- |
| 🔵 `NFT` or 🟣 `pNFT` | 🔵 🟣 | 🟣 | 🟣 | 🟣 | 🟣 | 🟣 | 🟣 (only once) | 🔵 |
| **Token State**        | 🔓 `Unlocked` | 🏠 `Listed` | 🔓 `Unlocked` | 🔐 `Locked`<br/>🔓 `Unlocked` | 🔐 `Locked`<br/>🔓 `Unlocked` | 🔐 `Locked`<br/>🔓 `Unlocked` | 🔐 `Locked`<br/>🔓 `Unlocked`| *Analogous to:* ❄️ `Frozen`<br/>☀️ `Thawn` |
| *Owner Transfer*     | ✅  | ❌ | ✅ → None | 🔓 if `Unlocked` → None | 🔓 if `Unlocked` → None |🔓 if Unlocked → None |🔓 if `Unlocked` → None|☀️ if `Thawn` → None|
| *Delegate Transfer*  | N/A | ✅ → None | ✅ → None | ✅ to locked address → None | ❌ | ❌ | 🔓 if `Unlocked` → None |☀️ if `Thawn` → None|
| *Owner Burn*         | ✅  | ❌ | ✅ | 🔓 if `Unlocked` | 🔓 if `Unlocked` | 🔓 if `Unlocked` | 🔓 if `Unlocked` | ☀️ if `Thawn` (full burn) |
| *Delegate Burn*      | N/A | ❌ | ❌ | ❌ | 🔓 if `Unlocked` | ❌ | 🔓 if `Unlocked` | ☀️ if `Thawn` (only SPL token) |
| *Owner Revoke*       | ❌  | ✅ → None | ✅ → None | 🔓 if `Unlocked` → None | 🔓 if `Unlocked` → None |🔓 if `Unlocked` → None|🔓 if `Unlocked` → None|☀️ if `Thawn`|
| *Owner Approve*      | ✅ → `Sale`, `Transfer`, `LockedTransfer`, `Staking` or `Utility` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ → `Standard` (SPL) |
| *Owner Unlock*       | ❌  | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| *Delegate Unlock*    | N/A | ❌ | ❌ | 🔐 if `Locked` | 🔐 if `Locked` | 🔐 if `Locked` | 🔐 if `Locked` | ☀️ if `Frozen` |
| *Owner Lock*         | ❌  | ❌ | ❌ | ❌ |  ❌ | ❌ | ❌ | ❌ |
| *Delegate Lock*      | N/A | ❌ | ❌ | 🔓 if `Unlocked` | 🔓 if `Unlocked` | 🔓 if `Unlocked` | 🔓 if `Unlocked` | ☀️ if `Thawn` |
| *Mint (destination)* | ✅  | ✅ | ✅ | 🔓 if `Unlocked` | 🔓 if `Unlocked` | 🔓 if `Unlocked` | 🔓 if `Unlocked` | ☀️ if `Thawn` |

The `Migration` delegate type is a temporary delegate that is only created by the migration from `NFT` to `pNFT` and cannot be otherwise created through the `Delegate` handler. This special delegate has the same functionality as the `Utility` delegate except that it can also transfer. This allows us to assign all escrowless-style programs this delegate to preserve whatever current functionality they have. Once used, it is cleared and cannot be replaced, and programs will then need to select one of the normal delegate types for future actions.

The `LockedTransfer` delegate type is a delegate that can lock and unlock a `pNFT` (similarly to the `Staking`) with the additional functionality of being able to transfer to a pre-determined address. The address is specified at the creation of the delegate through the `locked_address` argument.

> **Note**
> Once a token delegate is set, it is not possible to set another one unless the current one is revoked.

#### Metadata Delegates

`MetadataDelegate`s are delegates that operate at the metadata level. These delegates are represented by `MetadataDelegateRecord` PDA (seeds `["metadata", program id, mint id, delegate role, update authority id, delegate id]`) and do not have an associated spl-token delegate. There can be multiple instances of the same delegate.
```rust
pub struct MetadataDelegateRecord {
    pub key: Key,
    pub bump: u8,
    pub mint: Pubkey,
    pub delegate: Pubkey,
    pub update_authority: Pubkey,
}
```

Currently, we have three types of metadata delegates:

- `Collection`: can set and verify NFTs to a collection.
- `Update`: can perform updates on the metadata account.
- `Use`: allows an Actor to "use" the asset and decrement the uses counter on-chain, which is how applications can implement specific limited or tracking behaviors.

| Delegate Type | `Collection` | `Use` | `Update` |
| --------------------- | --- | --- | --- |
| *Delegate Collection* | ✅ | ❌ | ❌ |
| *Delegate Use*        | ❌ | ✅ | ❌ |
| *Delegate Update*     | ❌ | ❌ | ✅ |


### Handling Auth Rule Set Updates with Delegates

**Problem:** When interacting with programs, `pNFT`s have a configurable rule set that indicates which programs are allowed to interact with the asset. Given that a rule set can be edited at any point, this can cause issues for programs when rules change after they have become a delegate. The end result of this is that a `pNFT` could end up “stuck” in a contract, since the auth rules may have changed and the program has not changed to accomplish the requirements to interact with the asset.

**Solution:** Rule sets are stored with a revision number associated – i.e., each time an edit is performed, a new revision of the rule set is created. When a delegate is set on a `pNFT`, the rule set revision on the `pNFT` will be “locked” at the current (latest) revision and it will remain locked until the `pNFT` is transferred or the delegate is revoked. This will guarantee that the delegated program will be able to interact with the `pNFT` – the revision at the delegate point will be used to validate the actions. The “lock” on the rule set revision will also be released when a `Transfer` happens, since the delegate information gets cleared, and any further interaction will then use the latest revision of the rule set.

## 📦  JS SDK

Token Metadata includes a low-level Solita-based SDK, which can be used to interact with the new API. The NPM package can be found [here](https://www.npmjs.com/package/@metaplex-foundation/mpl-token-metadata/v/2.7.0).

The latest release of the [Metaplex JS SDK v0.18.0](https://github.com/metaplex-foundation/js#programmable-nfts) adds support for Programmable NFTs.

## 🏛️  Token Authorization Rules

There is a separate Token Authorization Rules program that provides the ability to create and execute rules to restrict the token operations discussed above.

### Overview

Authorization rules are variants of a `Rule` enum that implements a `validate()` function.

There are **Primitive Rules** and **Composed Rules** that are created by combining of one or more primitive rules:

- **Primitive Rules:** store any accounts or data needed for evaluation, and at runtime will produce a `true` or `false` output based on accounts and a well-defined `Payload` that are passed into the `validate()` function.
- **Composed Rules:** return a `true` or `false` based on whether any or all of the primitive rules return `true`.  Composed rules can then be combined into higher-level composed rules that implement more complex boolean logic.  Because of the recursive definition of the `Rule` enum, calling `validate()` on a top-level composed rule will start at the top and validate at every level, down to the component primitive rules.

More details of the Token Authorization Rules program, including examples, can be found [here](https://github.com/metaplex-foundation/mpl-token-auth-rules/blob/main/README.md).

### Token Metadata Operations subject to Authorization Rules

Several operations involving `pNFT` on Token Metadata are subject to Token Authorization Rules – depending on the rule configured, the operation will be authorized or not. The creator (`update authority`) of an asset has the flexibility to manage these rules through the [`ProgrammableConfig`](https://github.com/metaplex-foundation/metaplex-program-library/blob/ad5f39c465676299951c91f8cf9216812b884531/token-metadata/program/src/state/metadata.rs#L364-L380) on a Metadata account.

The definition of an operation follows a pattern `Operation:Scenario`, where `Operation` is the top-level action being performed and `Scenario` is a sub-categorization of the operation type. For example, in the case of `Transfer:Owner`, the top-level action is a `Transfer` being performed by the `Owner`.

The list of operations used in Token Metadata are:

- `Transfer:WalletToWallet`: operation representing a transfer between wallets (currently not in use)
- `Transfer:Owner`: operation representing a transfer initiated by the owner of the asset
- `Transfer:MigrationDelegate`: operation representing a transfer initiated by a `Migration` delegate
- `Transfer:SaleDelegate`: operation representing a transfer initiated by a `Sale` delegate
- `Transfer:TransferDelegate`: operation representing a transfer initiated by a `Transfer` delegate
- `Delegate:LockedTransfer`: operation representing the request to approve a `LockedTransfer` delegate, a delegate that can locked and transfer an asset to a predefined address
- `Delegate:Transfer`: operation representing the request to approve a `Transfer` delegate, a delegate that can transfer an asset
- `Delegate:Utility`: operation representing the request to approve a `Utility` delegate, a delegate that can lock and burn an asset
- `Delegate:Staking`: operation representing the request to approve a `Staking` delegate, a delegate that can lock an asset
- `Delegate:Sale`: operation representing the request to approve a `Sale` delegate, a delegate that can transfer an asset and disable transfers by the owner while the delegate is in place

> **Note:**
> When creating a custom rule set, it is important to include the operations above so Token Metadata functionality is not restricted. An operation that is not included in the rule set will always be denied.

## 🎬  Local Setup for Testing

The repository contains both Rust BPF and JavaScript/TypeScript. In order to setup the environment to run the tests, you will need to first clone the required repositories:

* Token Metadata: `https://github.com/metaplex-foundation/metaplex-program-library.git` branch `master`
* Token Authorization Rules: `https://github.com/metaplex-foundation/mpl-token-auth-rules.git`
* Rooster (for BPF tests): `https://github.com/metaplex-foundation/rooster`

This guide will assume that these repositories were cloned into a folder `$PROJECTS`.

### BPF tests (Rust)

To get Rust BPF tests working, you will first need to build both Token Auth Rules and Rooster programs:

* In the folder `$PROJECTS/mpl-token-auth-rules/program` execute:
  ```
  cargo build-bpf
  ```
  Then, copy the generated `.so` file from `$PROJECTS/mpl-token-auth-rules/program/target/deploy` into `$PROJECTS/metaplex-program-library/token-metadata/target/deploy/`
  
* In the folder `$PROJECTS/rooster/program` execute:
  ```
  cargo build-bpf
  ```
  Then, copy the generated `.so` file from `$PROJECTS/rooster/program/target/deploy` into `$PROJECTS/metaplex-program-library/token-metadata/target/deploy/`

> **Note:**
> The folder `$PROJECTS/metaplex-program-library/token-metadata/target/deploy/` might not exist. In this case, you will first need to build the token metadata program.

After building the programs, the BPF tests can be run from the folder `$PROJECTS/rooster/program/target/deploy` into `$PROJECTS/metaplex-program-library/token-metadata/program` by executing:
```
cargo test-bpf
```

### JavaScript/TypeScript tests

The JavaScript/TypeScript use [Amman](https://github.com/metaplex-foundation/amman) to start a local validator. The first step required is to build the required programs:

* In the folder `$PROJECTS/mpl-token-auth-rules/program` execute:
  ```
  cargo build-bpf
  ```
  Then, copy the generated `.so` file from `$PROJECTS/mpl-token-auth-rules/program/target/deploy` into `$PROJECTS/metaplex-program-library/test-programs/`
  
* In the folder `$PROJECTS/metaplex-program-library/` execute:
  ```
  ./build.sh token-metadata
  ```
  This will compile Token Metadata and copy the `.so` file into `$PROJECTS/metaplex-program-library/test-programs/`.
  
Then, you will need to navigate to the folder `$PROJECTS/metaplex-program-library/token-metadata/js/` and install the required dependencies:
```
yarn install
```
After all dependencies are installed, open a new terminal and start the Amman process. In the folder `$PROJECTS/metaplex-program-library/token-metadata/js/` execute:
```
yarn amman:start
```
The output will be similar to:
```
$ amman start
Loading config from /Users/febo/Projects/metaplex-program-library/token-metadata/js/.ammanrc.js
Running validator with 2 custom program(s) and 0 remote account(s) preloaded
Launching new solana-test-validator with programs predeployed and ledger at /var/folders/xr/5y4m3g8s49qgwr1r88ctz00m0000gn/T/amman-ledger
Successfully launched Relay at :::50474
ws error: connect ECONNREFUSED 127.0.0.1:8900
Successfully launched MockStorageServer at :::50475
ws error: connect ECONNREFUSED 127.0.0.1:8900
up and running
Waiting for fees to stabilize 1...
```
In your second terminal, navigate to the folder `$PROJECTS/metaplex-program-library/token-metadata/js/` and execute:
```
yarn build && yarn test
```
