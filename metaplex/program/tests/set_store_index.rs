#![cfg(any(test, feature = "test-bpf"))]
#![cfg_attr(not(feature = "test-bpf"), allow(dead_code))]

use std::{collections::HashMap, convert::TryInto};

use borsh::BorshSerialize;
use mpl_metaplex::{
    error::MetaplexError,
    id,
    instruction::{MetaplexInstruction, SetStoreIndexArgs},
    state::{
        AuctionCache, Key, Store, StoreIndexer, CACHE, INDEX, MAX_AUCTION_CACHE_SIZE,
        MAX_STORE_INDEXER_SIZE, MAX_STORE_SIZE, PREFIX,
    },
};
use solana_program::{
    account_info::AccountInfo, decode_error::DecodeError, program_error::ProgramError,
};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signer::Signer,
    transaction::Transaction,
};

/// Pretty-print a Metaplex program error
fn pretty_err(e: ProgramError) -> String {
    if let ProgramError::Custom(c) = e {
        if let Some(e) =
            <MetaplexError as DecodeError<MetaplexError>>::decode_custom_error_to_enum(c)
        {
            e.to_string()
        } else {
            e.to_string()
        }
    } else {
        e.to_string()
    }
}

/// One-stop shop for constructing a PDA account with an expected length
fn make_pda(seeds: &[&[u8]], acct: impl BorshSerialize, alloc_len: usize) -> (Pubkey, Account) {
    let mut data = vec![0_u8; alloc_len];
    // Borrow as a slice to impose a fixed allocation length
    acct.serialize(&mut data.as_mut_slice()).unwrap();

    (Pubkey::find_program_address(seeds, &id()).0, Account {
        lamports: 1_000_000_000,
        data,
        owner: id(),
        executable: false,
        rent_epoch: 0,
    })
}

/// Derive and serialize a `Store` into an account
fn make_store(owner: Pubkey, store: Store) -> (Pubkey, Account) {
    let id = id();

    make_pda(
        &[PREFIX.as_bytes(), id.as_ref(), owner.as_ref()],
        store,
        MAX_STORE_SIZE,
    )
}

/// Derive and serialize an `AuctionCache` into an account
fn make_cache(cache: AuctionCache) -> (Pubkey, Account) {
    let id = id();
    let store = cache.store;
    let auction = cache.auction;

    make_pda(
        &[
            PREFIX.as_bytes(),
            id.as_ref(),
            store.as_ref(),
            auction.as_ref(),
            CACHE.as_bytes(),
        ],
        cache,
        MAX_AUCTION_CACHE_SIZE,
    )
}

/// Derive and serialize a `StoreIndexer` into an account
fn make_index(index: StoreIndexer) -> (Pubkey, Account) {
    let id = id();
    let store = index.store;
    let page = index.page.to_string();

    make_pda(
        &[
            PREFIX.as_bytes(),
            id.as_ref(),
            store.as_ref(),
            INDEX.as_bytes(),
            page.as_bytes(),
        ],
        index,
        MAX_STORE_INDEXER_SIZE,
    )
}

/// A string, used here to quickly identify auction caches
type CacheId = &'static str;

/// A timestamp for an auction cache
struct Time(i64);

/// Simplification of a `SetStoreArgs` instruction for the test bed below
struct SetStoreIndex {
    offset: usize,
    cache: &'static str,
    above: Option<&'static str>,
    below: Option<&'static str>,
}

/// Test bed for `set_store_index` tests.
///
/// `caches` is a list of auction caches to be added to a new store index.
/// `extra_caches` is a list of auction caches which will be created but not
/// indexed, to be referenced by the test itself.  `args` is a sequence of
/// `SetStoreIndex` instructions to be processed, after which point the
/// modified store index will be compared against the list of keys in
/// `expected caches`.
async fn test_set_index<E: ExactSizeIterator<Item = CacheId>>(
    store_owner: Pubkey,
    caches: impl IntoIterator<Item = (CacheId, Time)>,
    extra_caches: impl IntoIterator<Item = (CacheId, Time)>,
    args: impl IntoIterator<Item = SetStoreIndex>,
    expected_caches: impl IntoIterator<IntoIter = E>,
) {
    let stub_key = Pubkey::new(&[0; 32]);
    let mut test = ProgramTest::new("mpl_metaplex", id(), None);

    let (store_key, store_acct) = make_store(store_owner, Store {
        key: Key::StoreV1,
        public: false,
        auction_program: stub_key,
        token_vault_program: stub_key,
        token_metadata_program: stub_key,
        token_program: stub_key,
    });
    test.add_account(store_key, store_acct);

    let mut cache_dict = HashMap::new();
    let mut auction_caches = vec![];

    for ((id, Time(timestamp)), index) in caches
        .into_iter()
        .map(|c| (c, true))
        .chain(extra_caches.into_iter().map(|e| (e, false)))
    {
        let (key, acct) = make_cache(AuctionCache {
            key: Key::AuctionCacheV1,
            store: store_key,
            timestamp,
            metadata: vec![],
            auction: stub_key,
            vault: stub_key,
            auction_manager: stub_key,
        });
        test.add_account(key, acct);
        assert!(
            cache_dict.insert(id, key).is_none(),
            "Duplicate cache ID {:?} given",
            id
        );

        if index {
            auction_caches.push(key);
        }
    }

    let page = 1337_u64;

    let (index_key, index_acct) = make_index(StoreIndexer {
        key: Key::StoreIndexerV1,
        store: store_key,
        page,
        auction_caches,
    });
    test.add_account(index_key, index_acct);

    let mut ctx = test.start_with_context().await;
    let payer = ctx.payer;
    let mut instructions = vec![];

    for SetStoreIndex {
        offset,
        cache,
        above,
        below,
    } in args
    {
        let mut accounts = vec![
            AccountMeta::new(index_key, false),
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new_readonly(cache_dict[cache], false),
            AccountMeta::new_readonly(store_key, false),
            AccountMeta::new_readonly(stub_key, false),
            AccountMeta::new_readonly(stub_key, false),
        ];

        accounts.extend(above.map(|a| AccountMeta::new_readonly(cache_dict[a], false)));
        accounts.extend(below.map(|b| AccountMeta::new_readonly(cache_dict[b], false)));

        for (i, account) in accounts.iter().enumerate() {
            if account.pubkey == stub_key || account.pubkey == payer.pubkey() {
                continue;
            }

            ctx.banks_client
                .get_account(account.pubkey)
                .await
                .unwrap()
                .expect(&format!(
                    "Passed nonexistant account argument {} at {}",
                    account.pubkey, i
                ));
        }

        instructions.push(Instruction {
            program_id: id(),
            accounts,
            data: MetaplexInstruction::SetStoreIndex(SetStoreIndexArgs {
                page,
                offset: offset.try_into().unwrap(),
            })
            .try_to_vec()
            .unwrap(),
        });
    }

    let tx = Transaction::new_signed_with_payer(
        &*instructions,
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let mut actual_index = ctx
        .banks_client
        .get_account(index_key)
        .await
        .unwrap()
        .expect("Missing store index after instructions");

    let actual_index = StoreIndexer::from_account_info(&AccountInfo::new(
        &index_key,
        false,
        false,
        &mut 1_000_000_000,
        &mut actual_index.data,
        &actual_index.owner,
        actual_index.executable,
        actual_index.rent_epoch,
    ))
    .map_err(pretty_err)
    .unwrap();

    let expected_caches = expected_caches.into_iter();
    assert_eq!(expected_caches.len(), actual_index.auction_caches.len());

    expected_caches
        .map(|c| cache_dict[c])
        .zip(actual_index.auction_caches.into_iter())
        .enumerate()
        .for_each(|(i, (expected, actual))| {
            assert_eq!(expected, actual, "Cache mismatch at index {}", i)
        });
}

mod set_store_index {
    use super::*;

    /// Assert that the test bench works at all
    #[cfg_attr(feature = "test-bpf", tokio::test)]
    async fn test_nop() {
        let store = Pubkey::new_unique();
        test_set_index(store, None, None, None, None).await;
    }

    /// Assert that the initial insertion into an index works
    #[cfg_attr(feature = "test-bpf", tokio::test)]
    async fn test_empty() {
        let store = Pubkey::new_unique();

        test_set_index(
            store,
            None,
            Some(("cache", Time(1))),
            Some(SetStoreIndex {
                offset: 0,
                cache: "cache",
                above: None,
                below: None,
            }),
            Some("cache"),
        )
        .await;
    }
}
