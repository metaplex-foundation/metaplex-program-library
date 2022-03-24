import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { MetadataProgram } from '../MetadataProgram';

export class UtilizeArgs extends Borsh.Data {
  static readonly SCHEMA = new Map([
    ...UtilizeArgs.struct([
      ['instruction', 'u8'],
      ['numberOfUses', 'u8'],
    ]),
  ]);
  instruction = 19;
  numberOfUses: number;
}

type UtilizeParams = {
  numberOfUses: number;
  metadata: PublicKey;
  mint: PublicKey;
  tokenAccount: PublicKey;
  owner: PublicKey;
  useAuthority?: PublicKey;
  burner?: PublicKey;
};

export class Utilize extends Transaction {
  constructor(options: TransactionCtorFields, params: UtilizeParams) {
    super(options);
    const { metadata, useAuthority, numberOfUses, burner, tokenAccount } = params;

    const data = UtilizeArgs.serialize({ numberOfUses });
    const accounts = [
      {
        pubkey: metadata,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: tokenAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: useAuthority,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: TOKEN_PROGRAM_ID,
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
    ];
    if (useAuthority) {
      accounts.push({
        pubkey: useAuthority,
        isSigner: false,
        isWritable: false,
      });
      accounts.push({
        pubkey: burner,
        isSigner: false,
        isWritable: false,
      });
    }
    this.add(
      new TransactionInstruction({
        keys: accounts,
        programId: MetadataProgram.PUBKEY,
        data,
      }),
    );
  }
}
