import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { Borsh } from '@metaplex/utils';
import { Transaction } from '../../../Transaction';
import { VaultProgram } from '../../vault';
import { MetadataProgram } from '../../metadata';
import { AuctionProgram } from '../../auction';
import { MetaplexProgram } from '../MetaplexProgram';
import { ParamsWithStore } from '@metaplex/types';

export class SetStoreArgs extends Borsh.Data<{ public: boolean }> {
  static readonly SCHEMA = this.struct([
    ['instruction', 'u8'],
    ['public', 'u8'],
  ]);

  instruction = 8;
  public: boolean;
}

type SetStoreParams = {
  admin: PublicKey;
  isPublic: boolean;
};

export class SetStore extends Transaction {
  constructor(options: TransactionCtorFields, params: ParamsWithStore<SetStoreParams>) {
    super(options);
    const { feePayer } = options;
    const { admin, store, isPublic } = params;

    const data = SetStoreArgs.serialize({ public: isPublic });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: store,
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
          { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
          {
            pubkey: VaultProgram.PUBKEY,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: MetadataProgram.PUBKEY,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: AuctionProgram.PUBKEY,
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
