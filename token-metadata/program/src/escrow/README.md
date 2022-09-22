# Bundt Cake (Composable NFTs via a Token Owned Escrow)
## Overview
This extension of the Token Metadata contract was created as a new feature primitive that could optionally be added to all NFTs. At its core it is simply an escrow account attached to an NFT, enabling NFTs to become owners of other tokens.

Aside from the requisite security and ownership checks necessary, the functionality this feature affords has been left generic enough to allow users to implement whatever they desire on top of the composability of the token and its escrow account. To give a better idea of what things can be accomplished, the [Using Bundt Cake](#using-bundt-cake) section provides several examples of implementations.
## Accounts
### Token Owned Escrow
The main account for this feature is the escrow account attached to the NFT. This can be considered the "wallet" that the NFT owns and uses to hold its tokens. This wallet has ownership over the various ATAs that are created to hold tokens transferred into it. Optionally, the Token Owned Escrow account also contains a pointer to a Constraint Model.
### Constraint Model
A Constraint Model is a set of restrictions and requirements that can be evaluated to allow for transmission into and out of the Token Owned Escrow account. On transfer, the contract will check against the constraint model to determine what checks need to be performed against the token being transferred to or from the TOE. One Constraint Model can serve many different NFTs and their TOE accounts.
## Instructions
### Create Escrow Account
Create the Token Owned Escrow account. This can only be performed on NFTs. Optionally, the Constraint Model can be set here.
### Close Escrow Account
Close the Token Owned Escrow account. A TOE must be empty in order to be closed to prevent loss of access to the tokens in escrow.
### Transfer In
Transfers a token into the TOE. Checks against the Constraint Model, if present.
### Transfer Out
Transfers a token out of the TOE. Checks against the Constraint Model, if present.
### Create Constraint Model
Creates a Constraint Model that can be used for Token Owned Escrow accounts.
### Add Constraint
Add a constraint to a Constraint Model.
## Constraint Types
### Metaplex Certified Collections
The token must belong to a specific Collection
### Token Set
The token mint must match one token in the set
## Using Bundt Cake
* Composable Images - The NFT Image is redrawn based on the tokens it contains in escrow (e.g. outfits, armor, layering images, composable attributes, etc.)
* Photo Filters - The base NFT has a specific filter and the image is drawn as the any NFT in the escrow account with that filter applied
* NFT ETFs - NFT assets are bundled together to be traded as bundles for DeFi