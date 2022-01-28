import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { MetadataProgram } from '../MetadataProgram';

export class VerifyCollectionArgs extends Borsh.Data {
  static readonly SCHEMA = new Map([...VerifyCollectionArgs.struct([['instruction', 'u8']])]);
  instruction = 18;
}

type VerifyCollectionParams = {
  metadata: PublicKey;
  collectionAuthorityRecord?: PublicKey;
  collectionAuthority: PublicKey;
  collectionMint: PublicKey;
  collectionMetadata: PublicKey;
  collectionMasterEdition: PublicKey;
};

export class VerifyCollection extends Transaction {
  constructor(options: TransactionCtorFields, params: VerifyCollectionParams) {
    super(options);
    const { feePayer } = options;
    const {
      metadata,
      collectionAuthority,
      collectionMint,
      collectionMetadata,
      collectionMasterEdition,
      collectionAuthorityRecord,
    } = params;

    const data = VerifyCollectionArgs.serialize();
    const accounts = [
      {
        pubkey: metadata,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: collectionAuthority,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: feePayer,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: collectionMint,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: collectionMetadata,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: collectionMasterEdition,
        isSigner: false,
        isWritable: false,
      },
    ];
    if (collectionAuthorityRecord) {
      accounts.push({
        pubkey: collectionAuthorityRecord,
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
