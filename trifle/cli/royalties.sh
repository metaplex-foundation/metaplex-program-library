#!/bin/bash
set -x

echo $'\n\n>>>Creating accounts'
KEY1=./1.json
solana-keygen new -s -o $KEY1 --no-bip39-passphrase --force
ADDR1=`solana-keygen pubkey $KEY1`

echo $'\n\n>>>Setting up accounts with NFTs and SOL'
solana airdrop -ul 2 "$ADDR1"

MODEL="test"
SLOT="first"

echo $'\n\n>>>Creating models with constraints'
ts-node src/trifle-cli.ts create model -r http://localhost:8899 -k "$KEY1" -n $MODEL -s none
ts-node src/trifle-cli.ts create constraint none -r http://localhost:8899 -k "$KEY1" -mn $MODEL -cn $SLOT -tl 1

echo $'\n\n>>>Setting up royalties'
ts-node src/trifle-cli.ts set_royalties create_trifle -r http://localhost:8899 -k "$KEY1" -mn $MODEL -c "$ADDR1" -f 0.001
ts-node src/trifle-cli.ts show model -r http://localhost:8899 -k "$KEY1" -n $MODEL | jq

echo $'\n\n>>>Changing royalties'
ts-node src/trifle-cli.ts set_royalties create_trifle -r http://localhost:8899 -k "$KEY1" -mn $MODEL -c "$ADDR1" -f 1
ts-node src/trifle-cli.ts show model -r http://localhost:8899 -k "$KEY1" -n $MODEL | jq