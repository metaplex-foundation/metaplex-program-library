import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { MetadataProgram } from '../MetadataProgram';

export class ApproveCollectionAuthorityArgs extends Borsh.Data {
  static readonly SCHEMA = new Map([
    ...ApproveCollectionAuthorityArgs.struct([['instruction', 'u8']]),
  ]);
  instruction = 23;
}

type ApproveCollectionAuthorityParams = {
  collectionAuthorityRecord: PublicKey;
  newCollectionAuthority: PublicKey;
  updateAuthority: PublicKey;
  metadata: PublicKey;
  mint: PublicKey;
};

export class ApproveCollectionAuthority extends Transaction {
  constructor(options: TransactionCtorFields, params: ApproveCollectionAuthorityParams) {
    super(options);
    const { feePayer } = options;
    const { metadata, collectionAuthorityRecord, newCollectionAuthority, updateAuthority, mint } =
      params;

    const data = ApproveCollectionAuthorityArgs.serialize();
    const accounts = [
      {
        pubkey: collectionAuthorityRecord,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: newCollectionAuthority,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: updateAuthority,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: feePayer,
        isSigner: true,
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
