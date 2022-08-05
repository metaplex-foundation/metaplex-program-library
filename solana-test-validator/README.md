# solana-test-validator

## Running test validator locally (using CLI)

### Instalation

Follow instructions at https://docs.solana.com/cli/install-solana-cli-tools

```sh
solana-test-validator --bpf-program hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk programs/mpl_auction_house.so
```

## Running test validator locally (using docker)

### Build locally

```sh
./scripts/dump-from-chain.sh
docker build -t solana-test-validator-base -f Dockerfile.base .
docker build -t solana-test-validator-onchain -f Dockerfile.onchain .
docker build -t solana-test-validator-source -f Dockerfile.source .
docker run -p 8899:8899 solana-test-validator-source
OR
docker run -p 8899:8899 solana-test-validator-onchain
```

### Use a prebuilt docker image

```sh
docker run ghcr.io/metaplex/solana-test-validator-onchain:master
OR
docker run ghcr.io/metaplex/solana-test-validator-source:master
```

## Airdropping some SOL

### Using CLI

```sh
solana airdrop 1 <RECIPIENT_ACCOUNT_ADDRESS> --url http://localhost:8899
```

### Using docker

```sh
docker run --network host solana solana airdrop 1 <RECIPIENT_ACCOUNT_ADDRESS> --url http://localhost:8899
```

## Compling programs

#### Auction program

```sh
cd metaplex-program-library/auction-house/program
cargo build-bpf --bpf-out-dir ../../../programs
```

## Dumping Data from Chain

You can take data directly from the chain and add it to the test validator.

### Dumping Programs

You can dump the BPF programs to a file from the chain instead of building them.

```
solana program dump hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk -u mainnet-beta mpl_auction_house.so
```

### Dumping Accounts

```
solana account -o 6dM4TqWyWJsbx7obrdLcviBkTafD5E8av61zfU6jq57X.json -u mainnet-beta 6dM4TqWyWJsbx7obrdLcviBkTafD5E8av61zfU6jq57X --output json
```
