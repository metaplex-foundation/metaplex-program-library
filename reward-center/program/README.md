# Reward Center

A program that facilitates the payout of a spl token as rewards to buyer and sellers on the successful sell of an NFTs.

## Definitions

reward center - A decorator for a Metaplex Auction Houses that owns the treasury of tokens to payout and manage reward rules. It is also the auctioneer delegate registered with the auction house program.

reward - Is the payout of some amount of tokens from the reward center treasury to the buyer and seller for the sell of an NFT.

reward rules - There are currently 2 configuration options the authority of the reward center can adjust to reward payout for a sale. They are the seller reward payout basis points and payout divider.

    payout divider - The amount to divide from the sale amount that will result in the number of tokens to payout to the buyer and the seller. For example, a divider of 2 will payout half the amount sale amount as tokens. Its important that the purchase and reward token use the same number of decimals to ensure the math aligns.

    seller reward payout basis points - The ratio of rewards to be sent to the seller. The rest of the rewards are claimed by the buyer. For example, 5,000 basis points will result in a 50-50 split of rewards to the buyer and the seller.




## Approach

The authority of an auction house can create a reward center to accompany it so tokens can be distributed when sales are brokered by the auction house. 

The reward center is the auctioneer delegate for the auction house of the auction house program.

Listing, offer, and purchase instructions destined for auction house are proxied through the reward center program so they can be enriched with rewards. Accounts are initialized for the listing and offers to track information needed to execute sales with auction house.

The authority of the reward center can control which NFTs receive rewards based on their associations to a Metaplex Collection. Rewardable collections are the on-chain record governing eligibility of an NFT for rewards. 

Through the auctioneer delegate feature of Auction House the reward center PDA is given authority over listings and offers ensuring any cancel requests go through the reward center program for documenting state changes.

## Instructions

### Create Reward Center

The authority of an auction house creates a reward center and sets the reward rules.

### Update Reward Center

The authority of an auction house with a reward center adjusts its configuration (e.g. collection oracle, reward rules).

### Create Listing

User puts an NFT up for sale through the reward center program. This results in a CPI call to the *sale* instruction of auction house. A listing record is generated to track sale order.

### Cancel Listing

User cancels their listing resulting in *cancel* CPI call to auction house and cancellation time saved on the listing.

### Update Listing

THe owner of a listing adjusts the sale price of the NFT.


### Create Offer

User places an offer on an NFT resulting in a *public_bid* CPI call to auction house and the creation of an offer account for the reward center. The amount of the offer is deducted from the user's wallet and placed in their escrow account.

### Cancel Offer

Users cancels their offer resulting in *cancel* CPI call to auction house and cancellation time saved on the offer. The amount of the offer is deducted from the user's escrow account and transferred back to the user's wallet.

### Execute Sale

Facilitates the sale of an NFT through a CPI call to auction house *execute_sale* and distributes rewards to the buyer and seller based on the configure reward rules by the auction house authority.
