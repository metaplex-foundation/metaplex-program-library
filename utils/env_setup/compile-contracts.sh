#!/bin/bash

# Metaplex Programs compilation for local development

# IMPORTANT! Make sure to run `cargo build` before running this shell script.
# Also if some package fails - delete `[package]/program/target directory``
# from package path and try again.

MPL_ROOT=$PROGRAM_ROOT
MPL_DEPLOY=${MPL_ROOT}/target/deploy

#replace_pubkey $keypath $replace_prefix $1

# Compile token-metadata program
source_path=/token-metadata/program
cd ${MPL_ROOT}${source_path}
cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
#anchor build

# Compile Auction program ------------------------------------------------------------------
source_path=/auction/program
cd ${MPL_ROOT}${source_path}
cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
#anchor build

# Compile Fixed Price Sale program
source_path=/fixed-price-sale/program
cd ${MPL_ROOT}${source_path}
#cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
anchor build

# Compile Metaplex
source_path=/metaplex/program
cd ${MPL_ROOT}${source_path}
cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
#anchor build

# Compile NFT-Packs program
source_path=/nft-packs/program
cd ${MPL_ROOT}${source_path}
cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
# anchor build

# Compile Entangler program
source_path=/token-entangler/program
cd ${MPL_ROOT}${source_path}
#cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
anchor build

# Compile token-vault program
source_path=/token-vault/program
cd ${MPL_ROOT}${source_path}
cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
# anchor build


#### PACKAGES WITH THEIR OWN target directories: -----------------------------------------

# Compile Auction House program ----------------------------------------------------------
source_path=/auction-house/program
cd ${MPL_ROOT}${source_path}
#cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
anchor build

# Compile Candy Machine program
# HAS OWN TARGET DIRECTORY
source_path=/candy-machine/program
cd ${MPL_ROOT}${source_path}
#cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
anchor build

# Compile Gumdrop program
source_path=/gumdrop/program
cd ${MPL_ROOT}${source_path}
#cargo build-bpf --bpf-out-dir ${MPL_DEPLOY}
anchor build