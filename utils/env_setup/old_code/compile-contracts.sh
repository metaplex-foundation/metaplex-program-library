#!/bin/bash

# Metaplex Programs compilation for local development

# IMPORTANT! Make sure to run `carbo build` before running this shell script.
# Also if some package fails - delete `[package]/program/target directory``
# from package path and try again.

MPL_ROOT=$PROGRAM_ROOT
MPL_DEPLOY=/target/deploy

replace_pubkey () {
  # $1-keypath, $2-replace_prefix, $3-input_file
  # check file exists
  if ! [ -f $1 ];
  then
    solana-keygen new --no-bip39-passphrase -o $keypath
  else
    echo "Already has key: $keypath"
  fi
	pubkey=$(solana address -k $1)
	echo "pubkey: $pubkey"
	searchstr="^$2\w\+\")"
	replacement="$2${pubkey}\")"
	#sed "s/${searchstr}/${replacement}/gm" $3 > $4
	sed -i "s/${searchstr}/${replacement}/gm" $3
}
#replace_pubkey $keypath $replace_prefix $1

# Compile token-metadata program
source_path=/token-metadata/program
cd ${MPL_ROOT}${source_path}
replace_prefix="solana_program::declare_id!(\""
keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_token_metadata-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
#anchor build

# Compile Auction program ------------------------------------------------------------------
source_path=/auction/program
cd ${MPL_ROOT}${source_path}
replace_prefix="solana_program::declare_id!(\""
keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_auction-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
#anchor build

# Compile Fixed Price Sale program
source_path=/fixed-price-sale/program
cd ${MPL_ROOT}${source_path}
replace_prefix="declare_id!(\""
keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_fixed_price_sale-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
#cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
anchor build

# Compile Metaplex
source_path=/metaplex/program
cd ${MPL_ROOT}${source_path}
replace_prefix="solana_program::declare_id!(\""
keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_metaplex-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
#anchor build

# Compile NFT-Packs program
source_path=/nft-packs/program
cd ${MPL_ROOT}${source_path}
replace_prefix="solana_program::declare_id!(\""
keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_nft_packs-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
# anchor build

# Compile Entangler program
source_path=/token-entangler/program
cd ${MPL_ROOT}${source_path}
replace_prefix="anchor_lang::declare_id!(\""
keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_token_entangler-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
#cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
anchor build

# Compile token-vault program
source_path=/token-vault/program
cd ${MPL_ROOT}${source_path}
replace_prefix="solana_program::declare_id!(\""
keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_token_vault-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
# anchor build


#### PACKAGES WITH THEIR OWN target directories: -----------------------------------------

# Compile Auction House program ----------------------------------------------------------
source_path=/auction-house/program
cd ${MPL_ROOT}${source_path}
replace_prefix="anchor_lang::declare_id!(\""
#keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_auction_house-keypair.json
keypath=${MPL_ROOT}${source_path}/target/deploy/mpl_auction_house-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
#cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
anchor build

# Compile Candy Machine program
# HAS OWN TARGET DIRECTORY
source_path=/candy-machine/program
cd ${MPL_ROOT}${source_path}
replace_prefix="anchor_lang::declare_id!(\""
#keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_candy_machine-keypair.json
keypath=${MPL_ROOT}${source_path}/target/deploy/mpl_candy_machine-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
#cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
anchor build

# Compile Gumdrop program
source_path=/gumdrop/program
cd ${MPL_ROOT}${source_path}
replace_prefix="declare_id!(\""
#keypath=${MPL_ROOT}${MPL_DEPLOY}/mpl_gumdrop-keypair.json
keypath=${MPL_ROOT}${source_path}/target/deploy/mpl_gumdrop-keypair.json
pwd
replace_pubkey $keypath $replace_prefix ${MPL_ROOT}${source_path}/src/lib.rs
#cargo build-bpf --bpf-out-dir ${MPL_ROOT}${MPL_DEPLOY}
anchor build