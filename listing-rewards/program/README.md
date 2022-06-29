# Listing Rewards

A program that facilitates the payout of a spl token as rewards for listing NFTS on a Metaplex Auction House for approved collections.

## Definitions

reward center - A decorator for a Metaplex Auction Houses that holds the treasury of tokens to payout as well as manage reward rules. It is also the auctioneer delegate registered with the auction house program.

reward - Is the payout of some amount of tokens from the reward center treasury to users that complete incentivized actions.

rewardable collection - Identifies which collections are eligable for a listing reward. Either the authority of the auction house or a collection oracle can adjust approved collections.

## Approach

The authority of an auction house can create a reward center to accompany it so tokens can be distributed when interacting with the auction house. 

The reward center is the auctioneer delegate for the auction house on the auction house program.

Listing, offer, and purchase instructions destined for auction house are proxied through the listing rewards program so they can be enriched with rewards. Accounts are initialized for the listing and offers to track information needed to execute sales with auction house as well as document potential rewards.

The authority of the reward center can control which NFTs receive rewards based on their associations to a Metaplex Collection. Rewardable collections are the on-chain record governing eligibility of an NFT for rewards. 

The authority of the reward center can delegate the management of rewardable collections to an oracle. The oracle is then responsible for maintaining the lis rewardable collections.

Through the auctioneer delegate feature of Auction House the reward center PDA will be given authority over listing and offers ensuring any cancel requests go through the listing rewards program for documenting state changes.

## Instructions

### Create Reward Center

The authority of an auction house creates a reward center.

### Update Reward Center

The authority of an auction house with a reward center adjusts its configuration (e.g. collection oracle, reward rules).

### Create Listing

User puts an NFT up for sale through the reward center program. This results in a CPI call to the *sale* instruction of auction house. A listing record is generated to track the listing and rewards.

### Cancel Listing

User cancels their listing resulting in *cancel* CPI call to auction house and cancellation time saved on the listing.

### Purchase Listing

User purchases a listed NFT resulting in *buy* and *execute sale* CPI call to auction house. The sales time of the listing is saved on the listing.

### Claim Listing Reward

The user can claim their tokens if the listing is past the warmup period or the listing was marked as “sold” by the reward center on the listing account.

### Create Offer

User places an offer on an NFT resulting in a *public_bid* CPI call to auction house and the creation of an offer account for the reward center.

### Accept Offer

Owner of an NFT accepts an offer resulting in *sell* and *execute sale* CPI calls to auction house. The time the offer was accepted is saved on the offer.

### Cancel Offer

Users cancels their offer resulting in *cancel* CPI call to auction house and cancellation time saved on the offer.

### Create Rewardable Collection

Make a collection eligible for rewards by saving account on-chain.

### Delete Rewardable Collection

Delete rewardable collection.
