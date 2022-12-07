# Token Metadata Escrow
## Overview
This extension of the Token Metadata contract was created as a new feature primitive that could optionally be added to all NFTs. At its core it is simply an escrow account attached to an NFT, enabling NFTs to become owners of other tokens.

Aside from the requisite security and ownership checks necessary, the functionality this feature affords has been left generic enough to allow users to implement whatever they desire on top of the composability of the token and its escrow account.
## Accounts
### Escrow
The main account for this feature is the escrow account attached to the NFT. This can be considered the "wallet" that the NFT owns and uses to hold its tokens. This wallet has ownership over the various ATAs that are created to hold tokens transferred into it.
## Instructions
### Create Escrow Account
Create the Token Owned Escrow account. This can only be performed on NFTs.
### Close Escrow Account
Close the Token Owned Escrow account.
### Transfer Out
Transfers a token out of the escrow account.

## Types of Escrow Accounts
### Token Owned Escrow
A Token Owned Escrow account, or TOE, is an escrow account attached the NFT that is managed by the holder of the NFT. Transferring a token out of this escrow account is only allowable by the tokens holder and the permissions follow the NFT as it is transferred between wallets. This means Alice can add a token to a TOE on her NFT, then sell her NFT to Bob. Bob would then be the only one allowed to transfer that token out of the TOE.
### Creator Owned Escrow
A Creator Owned Escrow, or COE, is an escrow account attached to the NFT that is managed by a specified creator. This escrow account allows creators to make associations between tokens that they themselves can manage, regardless of sales, transfers, and holders of the base NFT. An example use case for this is Metaverse avatars. Rather than storing avatars on a Web2 server, the Metaverse team could mint the avatar for an NFT as its own NFT, then put it in a COE (that the Metaverse team manages) attached to the corresponding NFT. Because usage of the COE is locked to the creator of the COE, a holder would be unable to transfer the avatar out of the escrow account and break the association.
