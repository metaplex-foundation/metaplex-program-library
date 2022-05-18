#!/bin/bash

# ! IMPORTANT ! - do not run manually. Designed to be called using `node change_pubkeys.js``

# $1-keypath
# check file exists
if ! [ -f $1 ];
then
  solana-keygen new --no-bip39-passphrase -o $1
else
  # echo "Already has key: $keypath"
  >&2 echo "Already has key: $1"
fi
pubkey=$(solana address -k $1)
echo $pubkey