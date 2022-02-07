import test from 'tape';
import { PayerTransactionHandler } from '@metaplex-foundation/amman';
import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';

import { createAndSignTransaction } from '../utils';
import { findSecondaryMetadataCreatorsAddress, CreatorAccountData } from '../../src';

import { createCreateSecondaryMetadataCreatorsInstruction } from '../../src/instructions/createSecondaryMetadataCreators';

type CreateSecondaryMetadataCreatorsParams = {
  test: test.Test;
  transactionHandler: PayerTransactionHandler;
  payer: Keypair;
  connection: Connection;
  metadata: PublicKey;
  creators: CreatorAccountData[];
};

export const createSecondaryMetadataCreators = async ({
  payer,
  connection,
  metadata,
  creators,
}: CreateSecondaryMetadataCreatorsParams): Promise<{
  createSecondaryMetadataCreatorsTx: Transaction;
  secondaryMetadataCreators: PublicKey;
}> => {
  const [secondaryMetadataCreators, secondaryMetadataCreatorsBump] =
    await findSecondaryMetadataCreatorsAddress(metadata);

  const createSecondaryMetadataCreatorsInstruction =
    createCreateSecondaryMetadataCreatorsInstruction(
      {
        admin: payer.publicKey,
        metadata,
        secondaryMetadataCreators,
      },
      {
        secondaryMetadataCreatorsBump,
        creators,
      },
    );

  const createSecondaryMetadataCreatorsTx = await createAndSignTransaction(
    connection,
    payer,
    [createSecondaryMetadataCreatorsInstruction],
    [payer],
  );

  return { createSecondaryMetadataCreatorsTx, secondaryMetadataCreators };
};
