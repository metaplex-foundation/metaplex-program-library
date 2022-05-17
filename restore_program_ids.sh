#!/bin/bash

# Deploy ALL Metaplex Programs
# Comment out any lines with programs you dont want to deploy.

# IMPORTANT! Make sure to follow instructions in LocalDev.md

# MPL_SO_PATH=/sol/metaplex/program-library/target/deploy
# MPL_WALLET=/wallet/metaplex.key.json

MPL_ROOT=/sol/metaplex/program-library

replace_pubkey () {
  # $1-pubkey, $2-replace_prefix, $3-input_file
  # check file exists
	echo "pubkey: $1"
	searchstr="^$2\w\+\")"
	replacement="$2${1}\")"
	#sed "s/${searchstr}/${replacement}/gm" $3 > $4
	sed -i "s/${searchstr}/${replacement}/gm" $3
}


metadata_arr=(/token-metadata/program "solana_program::declare_id!(\"" metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s)
auction_arr=(/auction/program "solana_program::declare_id!(\"" auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8)
house_arr=(/auction-house/program "anchor_lang::declare_id!(\"" hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk)
candy_arr=(/candy-machine/program "anchor_lang::declare_id!(\"" cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ)
fixed_arr=(/fixed-price-sale/program "declare_id!(\"" SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo)
plex_arr=(/metaplex/program "solana_program::declare_id!(\"" p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98)
pack_arr=(/nft-packs/program "solana_program::declare_id!(\"" packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu)
entangler_arr=(/token-entangler/program "anchor_lang::declare_id!(\"" qntmGodpGkrM42mN68VCZHXnKqDCT8rdY23wFcXCLPd)
gumdrop_arr=(/gumdrop/program "declare_id!(\"" gdrpGjVffourzkdDRrQmySw4aTHr8a3xmQzzxSwFD1a)
vault_arr=(/token-vault/program "solana_program::declare_id!(\"" vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn)

arr_list=(metadata_arr auction_arr house_arr candy_arr fixed_arr plex_arr pack_arr entangler_arr gumdrop_arr vault_arr)
arr=(arr_list)

declare -n elm1 elm2

for elm1 in "${arr[@]}"; do
  for elm2 in "${elm1[@]}"; do
      echo ${elm2[0]} # path
      echo ${elm2[1]} # prefix
      echo ${elm2[2]} # pubkey
      echo ""
      replace_pubkey ${elm2[2]} ${elm2[1]} ${MPL_ROOT}${elm2[0]}/src/lib.rs
  done
done



# path_pubkey[/token-metadata/program]=metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s

# for key in "${!path_pubkey[@]}"; do
#   echo "$key ${path_pubkey[$key][0]}"
# done

# auction
# auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8

# auction-house
# hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk

# candy machine
# cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ

# fixed-price-sale
# SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo

# gumdrop
# // declare_id!("gdrpGjVffourzkdDRrQmySw4aTHr8a3xmQzzxSwFD1a"); // original

# metaplex
# // solana_program::declare_id!("p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98"); // original

# nft-packs
# // solana_program::declare_id!("packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu") // original;

# token-entangler
# // anchor_lang::declare_id!("qntmGodpGkrM42mN68VCZHXnKqDCT8rdY23wFcXCLPd"); // original

# token-metadata
# // solana_program::declare_id!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"); // original

# token-vault
# // solana_program::declare_id!("vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn"); // original