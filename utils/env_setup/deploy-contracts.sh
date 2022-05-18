#!/bin/bash

# Deploy ALL Metaplex Programs
# Comment out any lines with programs you dont want to deploy.

# IMPORTANT! Make sure to follow instructions in LocalDev.md
MPL_WALLET=/wallet/metaplex.key.json

BASE_PATH=$PROGRAM_ROOT
MPL_SO_PATH=${BASE_PATH}/target/deploy
POSTFIX_PATH=/program/target/deploy

solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_token_metadata.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_auction.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_fixed_price_sale.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_metaplex.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_nft_packs.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_token_entangler.so
solana program deploy -k ${MPL_WALLET} ${MPL_SO_PATH}/mpl_token_vault.so

# PACKAGES with SO files inside their own target directories
target_path=${BASE_PATH}/auction-house${POSTFIX_PATH}
solana program deploy -k ${MPL_WALLET} ${target_path}/mpl_auction_house.so

target_path=${BASE_PATH}/candy-machine${POSTFIX_PATH}
solana program deploy -k ${MPL_WALLET} ${target_path}/mpl_candy_machine.so

target_path=${BASE_PATH}/gumdrop${POSTFIX_PATH}
solana program deploy -k ${MPL_WALLET} ${target_path}/mpl_gumdrop.so