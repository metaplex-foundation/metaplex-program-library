import { Borsh } from '@metaplex/utils';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
import { Transaction } from '../../../Transaction';
import { MetadataProgram } from '../MetadataProgram';

export class CreateMasterEditionArgs extends Borsh.Data<{ maxSupply: BN | null }> {
  static readonly SCHEMA = this.struct([
    ['instruction', 'u8'],
    ['maxSupply', { kind: 'option', type: 'u64' }],
  ]);

  instruction = 10;
  maxSupply: BN | null;
}

type CreateMasterEditionParams = {
  edition: PublicKey;
  metadata: PublicKey;
  updateAuthority: PublicKey;
  mint: PublicKey;
  mintAuthority: PublicKey;
  maxSupply?: BN;
};

export class CreateMasterEdition extends Transaction {
  constructor(options: TransactionCtorFields, params: CreateMasterEditionParams) {
    super(options);
    const { feePayer } = options;
    const { edition, metadata, updateAuthority, mint, mintAuthority, maxSupply } = params;

    const data = CreateMasterEditionArgs.serialize({
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
        ],
        programId: MetadataProgram.PUBKEY,
        data,
      }),
    );
  }
}
