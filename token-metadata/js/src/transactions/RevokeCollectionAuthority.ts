import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { MetadataProgram } from '../MetadataProgram';

export class RevokeCollectionAuthorityArgs extends Borsh.Data {
  static readonly SCHEMA = new Map([
    ...RevokeCollectionAuthorityArgs.struct([['instruction', 'u8']]),
  ]);
  instruction = 24;
}

type RevokeCollectionAuthorityParams = {
  collectionAuthorityRecord: PublicKey;
  newCollectionAuthority: PublicKey;
  updateAuthority: PublicKey;
  metadata: PublicKey;
  mint: PublicKey;
};

export class RevokeCollectionAuthority extends Transaction {
  constructor(options: TransactionCtorFields, params: RevokeCollectionAuthorityParams) {
    super(options);
    const { metadata, collectionAuthorityRecord, newCollectionAuthority, updateAuthority, mint } =
      params;

    const data = RevokeCollectionAuthorityArgs.serialize();
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
