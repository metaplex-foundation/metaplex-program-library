# Script pulling Solana programs from mainnet. Intended to be executed
# in /solana-test-validator before building the onchain image.
mkdir ./programs
solana program dump hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk ./programs/mpl_auction_house.so -u mainnet-beta
solana program dump p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98 ./programs/mpl_metaplex.so -u mainnet-beta
solana program dump vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn ./programs/mpl_token_vault.so -u mainnet-beta
solana program dump metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s ./programs/mpl_token_metadata.so -u mainnet-beta
mkdir ./metaplex-program-library/target
mkdir ./metaplex-program-library/target/idl
cd ./metaplex-program-library
RUST_BACKTRACE=1 anchor idl fetch hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk --provider.cluster mainnet | tee ./target/idl/auction_house.json
cd ..