#!/bin/bash

set -euox
main() {
    #
    # Build new keypair
    #
    solana-keygen new -s -o ../../target/deploy/mpl_auction_house-keypair.json --no-bip39-passphrase | true
    solana-keygen new -s -o ../../target/deploy/mpl_token_metadata-keypair.json --no-bip39-passphrase | true
    
    #
    # Build programs.
    #
    AUCTION_HOUSE_PID=$(solana address -k ../../target/deploy/mpl_auction_house-keypair.json)
    TOKEN_METADATA_PID=$(solana address -k ../../target/deploy/mpl_token_metadata-keypair.json)

    export AUCTION_HOUSE_PID
    export TOKEN_METADATA_PID
    #
    # Bootup validator.
    #
    solana-test-validator -r \
				--bpf-program $AUCTION_HOUSE_PID ../../target/deploy/mpl_auction_house.so \
				--bpf-program $TOKEN_METADATA_PID ../../target/deploy/mpl_token_metadata.so \
				> test-validator.log &
    sleep 5

    #
    # Run Test.
    #
    cargo test --test create_auction_house --test update_auction_house --test deposit --test withdraw \
                --test withdraw_from_fee --test withdraw_from_treasury
}

cleanup() {
    pkill -P $$ || true
    wait || true
}

trap_add() {
    trap_add_cmd=$1; shift || fatal "${FUNCNAME} usage error"
    for trap_add_name in "$@"; do
        trap -- "$(
            extract_trap_cmd() { printf '%s\n' "${3:-}"; }
            eval "extract_trap_cmd $(trap -p "${trap_add_name}")"
            printf '%s\n' "${trap_add_cmd}"
        )" "${trap_add_name}" \
            || fatal "unable to add to trap ${trap_add_name}"
    done
}

declare -f -t trap_add
trap_add 'cleanup' EXIT
main
