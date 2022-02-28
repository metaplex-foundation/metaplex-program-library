import { Connection, Keypair } from '@solana/web3.js';
import { TransactionHandler } from '@metaplex-foundation/amman';
import { DataV2 } from '../../src/accounts';
import { NAME, SELLER_FEE_BASIS_POINTS, SYMBOL, URI } from '../utils';
import { createMasterEdition } from './create-metadata';

export async function createCollection(
  connection: Connection,
  transactionHandler: TransactionHandler,
  payer: Keypair,
) {
  const initMetadataData = new DataV2({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
    collection: null,
    uses: null,
  });
  return await createMasterEdition(connection, transactionHandler, payer, initMetadataData, 0);
}
