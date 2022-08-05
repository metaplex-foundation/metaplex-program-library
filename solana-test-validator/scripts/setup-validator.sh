#!/bin/bash -ex

# Create test ledger on root directory, upload programs and all accounts from the /accounts folder
cd /

accounts=(/accounts/*.json)

#cat "${accounts[@]}"

command='solana-test-validator
    -u http://localhost:8899
    --bpf-program hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk /programs/mpl_auction_house.so
    --bpf-program p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98 /programs/mpl_metaplex.so
    --bpf-program vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn  /programs/mpl_token_vault.so
    --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s /programs/mpl_token_metadata.so '


#echo ${accounts[@]}
for name in ${accounts[@]}
do
    basename="${name##*/}"
    stripped="${basename%.*}"
    command=$command'    --account '$stripped' '$name' '
done

command="$command &"
echo -e $command

eval $command

# Allow time for validator to start
npx wait-on http://localhost:8899/health

cd /metaplex-program-library

anchor idl init \
    --provider.cluster http://localhost:8899 \
    --provider.wallet /keys/id.json \
    -f /metaplex-program-library/target/idl/auction_house.json \
    hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk

# Airdrop solanas to create idl and auction house accounts
solana airdrop --url http://localhost:8899 100

# Airdrop some solana to the auction house fee account
solana airdrop 100 8xTh4YhqaeRiybNpNKm2FyxTYmJtpRd2HgXGX7N74kH8 --url http://localhost:8899 

# Allow for idl to be initialized the account address is a PDA
/scripts/wait-solana-account.sh 9n1S8BHHCZi9GzbQvK3HcNbTTqtsiYbqnoSSD2y2et5q
sleep 10

# Create auction house
ts-node /metaplex/src/auction-house-cli.ts \
    create_auction_house -sfbp 250 -rso true --keypair /keys/id.json -e localnet

# Allow for test auction house creation, the account address is a PDA
/scripts/wait-solana-account.sh AMmjEEez3VumhAwFgEotRXvZtuvUWoPYkuy4CwkPBXe5
