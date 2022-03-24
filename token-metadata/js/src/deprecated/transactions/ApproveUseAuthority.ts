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

export class ApproveUseAuthorityArgs extends Borsh.Data {
  static readonly SCHEMA = new Map([
    ...ApproveUseAuthorityArgs.struct([
      ['instruction', 'u8'],
      ['numberOfUses', 'u8'],
    ]),
  ]);
  instruction = 20;
  numberOfUses: number;
}

type ApproveUseAuthorityParams = {
  useAuthorityRecord: PublicKey;
  user: PublicKey;
  owner: PublicKey;
  ownerTokenAccount: PublicKey;
  metadata: PublicKey;
  mint: PublicKey;
  burner: PublicKey;
  numberOfUses: number;
};

export class ApproveUseAuthority extends Transaction {
  constructor(options: TransactionCtorFields, params: ApproveUseAuthorityParams) {
    super(options);
    const { feePayer } = options;
    const {
      useAuthorityRecord,
      user,
      owner,
      ownerTokenAccount,
      metadata,
      mint,
      burner,
      numberOfUses,
    } = params;

    const data = ApproveUseAuthorityArgs.serialize({ numberOfUses });
    const accounts = [
      {
        pubkey: useAuthorityRecord,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: owner,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: feePayer,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: user,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: ownerTokenAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: metadata,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: mint,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: burner,
        isSigner: false,
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

    this.add(
      new TransactionInstruction({
        keys: accounts,
        programId: MetadataProgram.PUBKEY,
        data,
      }),
    );
  }
}
