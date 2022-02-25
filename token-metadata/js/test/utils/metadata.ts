import test from 'tape';
import spok from 'spok';

import { strict as assert } from 'assert';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { MetadataData, MetadataDataData } from '../../src/mpl-token-metadata';
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

  const initMetadataData = new MetadataDataData({
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
    initMetadataData,
  );
  const initialMetadata = await getMetadataData(connection, metadata);
  return { connection, transactionHandler, payer, mint, metadata, initialMetadata };
}

export async function getMetadataData(
  connection: Connection,
  metadata: PublicKey,
): Promise<MetadataData> {
  const metadataAccount = await connection.getAccountInfo(metadata);
  assert(metadataAccount != null, 'should find metadata account');
  return MetadataData.deserialize(metadataAccount.data);
}

/* eslint-disable @typescript-eslint/ban-ts-comment */
export async function assertMetadataDataUnchanged(
  t: test.Test,
  initial: MetadataData,
  updated: MetadataData,
  except?: keyof MetadataData,
) {
  const x = { ...initial };
  if (except != null) {
    delete x[except];
  }
  // @ts-ignore serves simpler test assertions
  delete x.data.creators;
  // @ts-ignore serves simpler test assertions
  delete x.tokenStandard;
  // @ts-ignore serves simpler test assertions
  delete x.collection;
  // @ts-ignore serves simpler test assertions
  delete x.uses;

  const y = { $topic: `no change except '${except}' on metadata`, ...updated };
  if (except != null) {
    delete y[except];
  }
  // @ts-ignore serves simpler test assertions
  delete y.data.creators;
  // @ts-ignore serves simpler test assertions
  delete y.tokenStandard;
  // @ts-ignore serves simpler test assertions
  delete y.collection;
  // @ts-ignore serves simpler test assertions
  delete y.uses;

  spok(t, x, y);
}

export async function assertMetadataDataDataUnchanged(
  t: test.Test,
  initial: MetadataDataData,
  updated: MetadataDataData,
  except: (keyof MetadataDataData)[],
) {
  const x = { ...initial };
  except.forEach((f) => delete x[f]);
  // @ts-ignore serves simpler test assertions
  delete x.creators;

  const y = { $topic: `no change except '${except}' on metadataData`, ...updated };
  except.forEach((f) => delete y[f]);
  // @ts-ignore serves simpler test assertions
  delete y.creators;

  spok(t, x, y);
}
/* eslint-enable @typescript-eslint/ban-ts-comment */
