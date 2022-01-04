import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { Data } from '../accounts/Metadata';
import { MetadataProgram } from '../MetadataProgram';

export class CreateMetadataArgs extends Borsh.Data<{ data: Data; isMutable: boolean }> {
  static readonly SCHEMA = new Map([
    ...Data.SCHEMA,
    ...CreateMetadataArgs.struct([
      ['instruction', 'u8'],
      ['data', Data],
      ['isMutable', 'u8'],
    ]),
  ]);

  instruction = 0;
  data: Data;
  isMutable: boolean;
}

type CreateMetadataParams = {
  pubkey: PublicKey;
  data: Data;
  updateAuthority: PublicKey;
  mint: PublicKey;
  mintAuthority: PublicKey;
};

export class CreateMetadata extends Transaction {
  constructor(options: TransactionCtorFields, params: CreateMetadataParams) {
    super(options);
    const { feePayer } = options;
    const { pubkey, data, updateAuthority, mint, mintAuthority } = params;

    const metadata = CreateMetadataArgs.serialize({
      data,
      isMutable: true,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: pubkey,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: mint,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: mintAuthority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: feePayer,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: updateAuthority,
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
        programId: MetadataProgram.PUBKEY,
        data: metadata,
      }),
    );
  }
}
