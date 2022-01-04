import test from 'tape';
import spok from 'spok';

import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { Metadata, Data } from '../../src/mpl-token-metadata';
import { connectionURL } from './';
import { airdrop, PayerTransactionHandler } from '@metaplex-foundation/amman';

import { addLabel } from './address-labels';
import { mintAndCreateMetadata } from '../actions';

export const URI = 'uri';
export const NAME = 'test';
export const SYMBOL = 'sym';
export const SELLER_FEE_BASIS_POINTS = 10;

export async function initMetadata() {
  const payer = Keypair.generate();
  addLabel('payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const initMetadata = new Data({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
  });

  const { mint, metadata } = await mintAndCreateMetadata(
    connection,
    transactionHandler,
    payer,
    initMetadata,
  );
  const initialMetadata = await getMetadata(connection, metadata);
  return { connection, transactionHandler, payer, mint, metadata, initialMetadata };
}

export async function getMetadata(
  connection: Connection,
  metadata: PublicKey,
): Promise<Metadata> {
  const metadataAccount = await connection.getAccountInfo(metadata);
  return Metadata.deserialize(metadataAccount.data);
}

export async function assertMetadataUnchanged(
  t: test.Test,
  initial: Metadata,
  updated: Metadata,
  except?: keyof Metadata,
) {
  const x = { ...initial };
  if (except != null) {
    delete x[except];
  }
  delete x.data.creators;

  const y = { $topic: `no change except '${except}' on metadata`, ...updated };
  if (except != null) {
    delete y[except];
  }
  delete y.data.creators;

  spok(t, x, y);
}

export async function assertDataUnchanged(
  t: test.Test,
  initial: Data,
  updated: Data,
  except: (keyof Data)[],
) {
  const x = { ...initial };
  except.forEach((f) => delete x[f]);
  delete x.creators;

  const y = { $topic: `no change except '${except}' on Metadata`, ...updated };
  except.forEach((f) => delete y[f]);
  delete y.creators;

  spok(t, x, y);
}
