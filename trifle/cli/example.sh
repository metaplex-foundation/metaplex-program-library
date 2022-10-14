#!/bin/bash
set -x
# Create Constraint Model
ts-node src/trifle-cli.ts create model -r http://localhost:8899 -k ~/.config/solana/id.json -n test -s none | jq
ts-node src/trifle-cli.ts create constraint none -r http://localhost:8899 -k ~/.config/solana/id.json -mn test -cn first | jq
ts-node src/trifle-cli.ts create constraint collection -r http://localhost:8899 -k ~/.config/solana/id.json -mn test -cn first -tl 1 -c 6XxjKYFbcndh2gDcsUrmZgVEsoDxXMnfsaGY6fpTJzNr | jq

# Create Trifle
ts-node src/trifle-cli.ts create trifle -r http://localhost:8899 -k ~/.config/solana/id.json -c -u https://arweave.net/LcjCf-NDr5bhCJ0YMKGlc8m8qT_J6TDWtIuW8lbu0-A -n BH1 -mn test | jq