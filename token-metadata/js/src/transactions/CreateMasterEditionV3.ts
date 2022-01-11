import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
import { CreateMasterEditionParams } from '.';
import { MetadataProgram } from '../MetadataProgram';

export class CreateMasterEditionV3Args extends Borsh.Data<{ maxSupply: BN | null }> {
  static readonly SCHEMA = CreateMasterEditionV3Args.struct([
    ['instruction', 'u8'],
    ['maxSupply', { kind: 'option', type: 'u64' }],
  ]);

  instruction = 17;
  maxSupply: BN | null;
}

export class CreateMasterEditionV3 extends Transaction {
  constructor(options: TransactionCtorFields, params: CreateMasterEditionParams) {
    super(options);
    const { feePayer } = options;
    const { edition, metadata, updateAuthority, mint, mintAuthority, maxSupply } = params;

    const data = CreateMasterEditionV3Args.serialize({
      maxSupply: maxSupply || null,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: edition,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: mint,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: updateAuthority,
            isSigner: true,
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
            pubkey: metadata,
            isSigner: false,
            isWritable: true,
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
        ],
        programId: MetadataProgram.PUBKEY,
        data,
      }),
    );
  }
}
