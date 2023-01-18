#!/bin/bash
set -x

echo $'\n\n>>>Creating accounts'
KEY1=./1.json
KEY2=./2.json
solana-keygen new -s -o $KEY1 --no-bip39-passphrase --force
ADDR1=`solana-keygen pubkey $KEY1`
solana-keygen new -s -o $KEY2 --no-bip39-passphrase --force
ADDR2=`solana-keygen pubkey $KEY2`

echo $'\n\n>>>Setting up accounts with NFTs and SOL'
solana airdrop -ul 2 "$ADDR1"
solana airdrop -ul 2 "$ADDR2"

NFT=`metaboss mint one -r http://localhost:8899 -k $KEY2 -u https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls | grep Mint | awk '{print $3}'`
ITEM=`metaboss mint one -r http://localhost:8899 -k $KEY2 -u https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls | grep Mint | awk '{print $3}'`


MODEL="test"
SLOT="first"

echo $'\n\n>>>Creating models with constraints'
ts-node src/trifle-cli.ts create model -r http://localhost:8899 -k "$KEY1" -n $MODEL -s none
ts-node src/trifle-cli.ts create constraint none -r http://localhost:8899 -k "$KEY1" -mn $MODEL -cn $SLOT -tl 1

echo $'\n\n>>>Creating trifle'
ts-node src/trifle-cli.ts create trifle -r http://localhost:8899 -k "$KEY1" -mn $MODEL -m $NFT

echo $'\n\n>>>Transferring item in'
ts-node src/trifle-cli.ts transfer in -r http://localhost:8899 -k "$KEY2" -mn $MODEL -m $NFT -am $ITEM -s $SLOT -a 1 -c "$ADDR1"

echo $'\n\n>>>Transferring item out'
ts-node src/trifle-cli.ts transfer out -r http://localhost:8899 -k "$KEY2" -mn $MODEL -m $NFT -am $ITEM -s $SLOT -a 1 -c "$ADDR1"
