import test from 'tape';

import { DataV2, MetadataDataData, UpdateMetadataV2, VerifyCollection } from '../src/mpl-token-metadata';
import {
    killStuckProcess,
    initMetadata,
    getMetadataData,
    assertMetadataDataUnchanged,
    assertMetadataDataDataUnchanged,
    URI,
    NAME,
    SYMBOL,
    connectionURL,
    SELLER_FEE_BASIS_POINTS,
} from './utils';
import { airdrop, assertError, PayerTransactionHandler, TransactionHandler } from '@metaplex-foundation/amman';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { mintAndCreateMetadata, createMasterEdition } from './actions';
import { Collection } from '../src/accounts';

killStuckProcess();

async function createCollection(
    connection: Connection,
    transactionHandler: TransactionHandler,
    payer: Keypair,
) {
    const initMetadataData = {
        uri: URI,
        name: NAME,
        symbol: SYMBOL,
        sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
        creators: null,
        collection: null,
        uses: null,
    };
    return await createMasterEdition(
        connection,
        transactionHandler,
        payer,
        initMetadataData,
    );
}


// -----------------
// Success Cases
// -----------------

test('update-metadata-account: toggle primarySaleHappened', async (t) => {
    const payer = Keypair.generate();
    const connection = new Connection(connectionURL, 'confirmed');
    const transactionHandler = new PayerTransactionHandler(connection, payer);

    await airdrop(connection, payer.publicKey, 2);

    let collectionNft = await createCollection(connection, transactionHandler, payer);

    const initMetadataData = {
        uri: URI,
        name: NAME,
        symbol: SYMBOL,
        sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
        creators: null,
        collection: new Collection({ key: collectionNft.mint.publicKey.toBase58(), verified: false }),
        uses: null,
    };
    let collectionMemberNft = await createMasterEdition(
        connection,
        transactionHandler,
        payer,
        initMetadataData,
    );

    const updatedMetadataBeforeVerification = await getMetadataData(connection, collectionMemberNft.metadata);
    t.ok(updatedMetadataBeforeVerification.collection, 'collection should be not null');
    t.not(updatedMetadataBeforeVerification.collection.verified, 'collection should be not be verified');

    const collectionVerifyCollectionTransaction = new VerifyCollection({}, {
        metadata: collectionMemberNft.metadata,
        collectionAuthority: payer.publicKey,
        collectionMint: collectionNft.mint.publicKey,
        collectionMetadata: collectionNft.metadata,
        collectionMasterEdition: collectionNft.masterEditionPubkey,
    })

    await transactionHandler.sendAndConfirmTransaction(collectionVerifyCollectionTransaction, [payer]);

    const updatedMetadata = await getMetadataData(connection, collectionMemberNft.metadata);

    const updatedMetadataAfterVerification = await getMetadataData(connection, collectionMemberNft.metadata);
    t.ok(updatedMetadataBeforeVerification.collection, 'collection should be not null');
    t.ok(updatedMetadataBeforeVerification.collection.verified, 'collection should be not be verified');

});
