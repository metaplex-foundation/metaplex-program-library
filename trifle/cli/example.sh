#!/bin/bash
set -x

# Test setup
# Create NFTs
BASE=`metaboss mint one -r http://localhost:8899 -k ~/.config/solana/id.json -u https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls | grep Mint | awk '{print $3}'`
NONE=`metaboss mint one -r http://localhost:8899 -k ~/.config/solana/id.json -u https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls | grep Mint | awk '{print $3}'`
COLLECTION=`metaboss mint one -r http://localhost:8899 -k ~/.config/solana/id.json -u https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls | grep Mint | awk '{print $3}'`
TOKEN=`metaboss mint one -r http://localhost:8899 -k ~/.config/solana/id.json -u https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls | grep Mint | awk '{print $3}'`
echo "[\"$TOKEN\"]" > tokens.json

# Create Constraint Model
ts-node src/trifle-cli.ts create model -r http://localhost:8899 -k ~/.config/solana/id.json -n test -s none | jq

ts-node src/trifle-cli.ts create constraint none -r http://localhost:8899 -k ~/.config/solana/id.json -mn test -cn first -tl 1 | jq
ts-node src/trifle-cli.ts create constraint collection -r http://localhost:8899 -k ~/.config/solana/id.json -mn test -cn second -tl 1 -c $COLLECTION | jq
ts-node src/trifle-cli.ts create constraint tokens -r http://localhost:8899 -k ~/.config/solana/id.json -mn test -cn third -tl 1 -p tokens.json | jq

# Create Trifle
ts-node src/trifle-cli.ts create trifle -r http://localhost:8899 -k ~/.config/solana/id.json -m $BASE -u https://arweave.net/LcjCf-NDr5bhCJ0YMKGlc8m8qT_J6TDWtIuW8lbu0-A -n BH1 -mn test | jq

# Transfer In Tokens
ts-node src/trifle-cli.ts transfer in -r http://localhost:8899 -k ~/.config/solana/id.json -m $BASE -mn test -am $NONE -a 1 -s first | jq
ts-node src/trifle-cli.ts transfer in -r http://localhost:8899 -k ~/.config/solana/id.json -m $BASE -mn test -am $TOKEN -a 1 -s third | jq

# Transfer Out Tokens
ts-node src/trifle-cli.ts transfer out -r http://localhost:8899 -k ~/.config/solana/id.json -m $BASE -mn test -am $NONE -a 1 -s first | jq
ts-node src/trifle-cli.ts transfer out -r http://localhost:8899 -k ~/.config/solana/id.json -m $BASE -mn test -am $TOKEN -a 1 -s third | jq