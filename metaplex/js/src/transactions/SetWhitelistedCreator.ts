import { ParamsWithStore } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { Transaction } from '../../../Transaction';
import { MetaplexProgram } from '../MetaplexProgram';

export class SetWhitelistedCreatorArgs extends Borsh.Data<{ activated: boolean }> {
  static readonly SCHEMA = this.struct([
    ['instruction', 'u8'],
    ['activated', 'u8'],
  ]);

  instruction = 9;
  activated: boolean;
}

type SetWhitelistedCreatorParams = {
  admin: PublicKey;
  whitelistedCreatorPDA: PublicKey;
  creator: PublicKey;
  activated: boolean;
};

export class SetWhitelistedCreator extends Transaction {
  constructor(
    options: TransactionCtorFields,
    params: ParamsWithStore<SetWhitelistedCreatorParams>,
  ) {
    super(options);
    const { feePayer } = options;
    const { admin, whitelistedCreatorPDA, store, creator, activated } = params;

    const data = SetWhitelistedCreatorArgs.serialize({ activated });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: whitelistedCreatorPDA,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: admin,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: feePayer,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: creator,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: store,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
          },
        ],
        programId: MetaplexProgram.PUBKEY,
        data,
      }),
    );
  }
}
