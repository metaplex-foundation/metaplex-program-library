/* eslint-disable @typescript-eslint/no-non-null-assertion */
import test from 'tape';
import { PayerTransactionHandler } from '@metaplex-foundation/amman-client';
import * as web3 from '@solana/web3.js';
import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';

import { createTokenAccount } from '../transactions';
import { createAndSignTransaction, logDebug } from '../utils';

import { findTreasuryOwnerAddress } from '../../src/utils';
import {
  createCreateMarketInstruction,
  CreateMarketInstructionArgs,
} from '../../src/generated/instructions';

type CreateMarketParams = {
  test: test.Test;
  transactionHandler: PayerTransactionHandler;
  payer: Keypair;
  connection: Connection;
  store: PublicKey;
  sellingResource: PublicKey;
  treasuryMint: PublicKey;
  collectionMint?: PublicKey;
  params: Omit<CreateMarketInstructionArgs, 'treasuryOwnerBump'>;
};

export const createMarket = async ({
  test,
  transactionHandler,
  payer,
  connection,
  store,
  sellingResource,
  treasuryMint,
  collectionMint,
  params,
}: CreateMarketParams): Promise<{
  market: Keypair;
  treasuryHolder: Keypair;
  treasuryOwnerBump: number;
  treasuryOwner: PublicKey;
}> => {
  const [treasuryOwner, treasuryOwnerBump] = await findTreasuryOwnerAddress(
    treasuryMint,
    sellingResource,
  );

  logDebug(`treasuryOwner: ${treasuryOwner.toBase58()}`);

  const { tokenAccount: treasuryHolder, createTokenTx } = await createTokenAccount({
    payer: payer.publicKey,
    connection,
    mint: treasuryMint,
    owner: treasuryOwner,
  });

  await transactionHandler
    .sendAndConfirmTransaction(createTokenTx, [treasuryHolder])
    .assertSuccess(test);
  logDebug(`treasuryHolder: ${treasuryHolder.publicKey}`);

  const market = Keypair.generate();

  const remainingAccounts: web3.AccountMeta[] = [];

  if (collectionMint) {
    remainingAccounts.push({ pubkey: collectionMint!, isWritable: true, isSigner: false });
  }

  const instruction = createCreateMarketInstruction(
    {
      market: market.publicKey,
      store,
      sellingResourceOwner: payer.publicKey,
      sellingResource,
      mint: treasuryMint,
      treasuryHolder: treasuryHolder.publicKey,
      owner: treasuryOwner,
      anchorRemainingAccounts: remainingAccounts,
    },
    {
      treasuryOwnerBump,
      ...params,
    },
  );

  const marketTx: Transaction = await createAndSignTransaction(
    connection,
    payer,
    [instruction],
    [market],
  );

  await transactionHandler.sendAndConfirmTransaction(marketTx, [market]).assertSuccess(test);
  logDebug(`market: ${market.publicKey}`);

  return { market, treasuryHolder, treasuryOwnerBump, treasuryOwner };
};
