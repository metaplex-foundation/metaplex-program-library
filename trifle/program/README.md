# Trifle (NFT Composability)
## Overview
Trifle is a program built upon the Escrow extension of Token Metadata. It uses a Creator Owned Escrow, or COE, using a Trifle PDA as the creator and manager of the COE. Its purpose is to add on-chain tracking and composability around NFT ownership. Additionally, the ability to specify rules around token ownership allows for complex ownership models to be implemented by creators.
## Accounts
### Escrow Constraint Model
A Constraint Model is a set of restrictions and requirements that can be evaluated to allow for transmission into and out of the Trifle account. On transfer, the contract will check against the constraint model to determine what checks need to be performed against the token being transferred to or from the TOE. One Constraint Model can serve many different NFTs and their Trifle accounts.

The Constraint Model account also functions as a treasury for royalties paid by users using Trifle. The royalties map manages the royalties paid by users for each instruction and the royalties balance shows the current total.
### Trifle
The Trifle account is what tracks tokens owned by the COE on-chain. It also links to the Constraint Model being used. The token tracking is done through a series of slots (identified using a string) that point to a vector of tokens.
## Instructions
### Create Escrow Constraint Model Account
Creates a Constraint Model that can be used for Trifle accounts.
### Create Trifle Account
### Transfer In
Transfer a token into the Creator Owned Escrow managed by the Trifle account. While it is possible to do a standard spl-token transfer to the COE, using this instruction is the only way for the Trifle account to manage and track the owned tokens. This instruction also performs checks against the Constraint Model to verify that the token being transferred in is valid.
### Transfer Out
Transfer a token out of the Creator Owned Escrow managed by the Trifle account. This instruction also performs checks against the Constraint Model to verify that the token being transferred out is allowed to be removed.
### Add Constraints
Add a constraint to a Constraint Model.
### Remove Constraint
Remove a constraint from the Constraint Model
### Set Royalties
Set royalties on Trifle instructions on a per-ix basis
### Withdraw Royalties
Withdraw royalties from the Constraint Model to the creator's wallet
## Constraint Types
### None
No requirements and allows transferring in any token with no limit.
### Metaplex Certified Collections
The token must belong to a specific Collection.
### Token Set
The token mint must match one token in the set.

## Royalties
The Trifle program allows for an alternate royalties revenue model where a project can charge royalties for Trifle interactions on any project that utilizes their Constraint Model. A map is stored in the Constraint Model that creates granular control of a royalty amount for each instruction. This allows creators to take optionally royalties for Trifle account creation, transferring in, or transferring out.

The royalties are then stored in the Constraint Model which doubles as a treasury account. Royalties can be withdrawn by the update authority/creator on the Constraint Model account.

## Protocol Fees
Metaplex charges minimal protocol fees for Trifle interactions. Creator creation is free with the creation of the Constraint Model having no associated fee. Further modifications require per-transaction 0.1 SOL fee. Standard Trifle functions have a 0.02 SOL associated protocol fee. 10% of the optional Creator royalties are also counted toward protocol fees.

## Using Trifle
* Composable Images - The NFT Image is redrawn based on the tokens it contains in escrow (e.g. outfits, armor, layering images, composable attributes, etc.)
* Photo Filters - The base NFT has a specific filter and the image is drawn as the any NFT in the escrow account with that filter applied
* NFT ETFs - NFT assets are bundled together to be traded as bundles for DeFi