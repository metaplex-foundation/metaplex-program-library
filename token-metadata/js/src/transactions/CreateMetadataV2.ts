import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { DataV2 } from '../accounts/Metadata';
import { MetadataProgram } from '../MetadataProgram';

export class CreateMetadataV2Args extends Borsh.Data<{ data: DataV2; isMutable: boolean }> {
  static readonly SCHEMA = new Map([
    ...DataV2.SCHEMA,
    ...CreateMetadataV2Args.struct([
      ['instruction', 'u8'],
      ['data', DataV2],
      ['isMutable', 'u8'],
    ]),
  ]);

  instruction = 16;
  data: DataV2;
  isMutable: boolean;
}

export type CreateMetadataV2Params = {
  metadata: PublicKey;
  metadataData: DataV2;
  updateAuthority: PublicKey;
  mint: PublicKey;
  mintAuthority: PublicKey;
};

export class CreateMetadataV2 extends Transaction {
  constructor(options: TransactionCtorFields, params: CreateMetadataV2Params) {
    super(options);
    const { feePayer } = options;
    const { metadata, metadataData, updateAuthority, mint, mintAuthority } = params;

    const data = CreateMetadataV2Args.serialize({
      data: metadataData,
      isMutable: true,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: metadata,
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
        data,
      }),
    );
  }
}
