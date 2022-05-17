#!/bin/bash

# Deploy ALL Metaplex Programs
# Comment out any lines with programs you dont want to deploy.

# IMPORTANT! Make sure to follow instructions in LocalDev.md

MPL_SO_PATH=/sol/metaplex/program-library/target/deploy
MPL_WALLET=/wallet/metaplex.key.json

solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_token_metadata.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_auction.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_auction_house.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_candy_machine.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_fixed_price_sale.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_metaplex.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_nft_packs.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_token_entangler.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_gumdrop.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_token_vault.so

