
# Running this thing locally

Recommend using or building your own solana development Docker container.
Install Docker, and docker-compose.
If you don't want to build your own dev container - you can use this one I built and
deployed on DockerHub if not have an AVX2 enabled CPU. NOT A PRODUCTION CONTAINER.
`dmitryr117/anchor-noavx2:0.24.2`

Using the following docker-compose configuration:

```
version: "3"

services:
  soldev:
    # IMPORTANT!: for local development make sure to turn on  
    image: dmitryr117/anchor-noavx2:0.24.2
    container_name: soldev
    restart: on-failure
    working_dir: /
    volumes:
      - ./appdev:/sol
      - ./devenv/wallet:/wallet
      - ./devenv/config/validator.yml:/root/.config/solana/cli/config.yml
      - ./devenv/validator/test-ledger:/test-ledger
    ports:
      - 8899:8899
```

Installing Solana programs:

1. `docker exec -ti soldev /bin/bash` into deployed Docker container

2. run `cd /sol/metaplex/program-library`

3. run `cargo build` This step is very important. Otherwise required packages will be missing.

4. run `./compile-contracts.sh` This will assign ...-keypair.json files to all smart 
   contracts if not exists, update `declare_id!()` values inside corresponding their **lib.rs** files 
   with new publick keys extracted from these keypairs, compile all contracts, and store them all and 
   their kaypairs in `./target/deploy`
   **IMPORTANT!** Script will not overwrite any **...keypair.json** files if they already exist in `./target/deploy`
   **Troubleshooting** Sometimes a build can fail complaining about some package missing. So for example if
   a package is missing or corrupted in `./gumdrop/program/target` - the package will not compile.
   In this case delete `./gumdrop/program/target` directory and its contents, go back to `/sol/metaplex/program-library`,
   and re-run `./compile-contracts.sh`

5. Create and fund wallet for uploading smart contracts.
   Inside container run `solana-keygen new --outfile /wallet/metaplex.key.json`

6. Open another terminal and log into same docker instance again using step **1** and **2**.

7. run `solana-test-validator` to start a local solana validator instance. Naw this trerminal will be showing block production, 
   and will remain blocked until `Ctrl-C` input.

8. Return to 1st terminal logged inside `soldev` container and fund the wallet with 100 SOL:
   `solana airdrop -k /wallet/metaplex.key.json 100`

9. Deploy smart contracts using `./deploy-contracts.sh` shell script.



<!-- 9. run ./anchor-predeploy.sh to copy all keys and compiled files into `./target/deploy` directory

10. `anchor deploy` to deploy packages in `./target/deploy`

These are required to complete full metaplex smart-comtract ecosystem setup. -->