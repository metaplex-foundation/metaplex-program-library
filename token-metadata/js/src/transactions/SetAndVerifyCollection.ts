import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { MetadataProgram } from '../MetadataProgram';

export class SetAndVerifyCollectionArgs extends Borsh.Data {
  static readonly SCHEMA = new Map([...SetAndVerifyCollectionArgs.struct([['instruction', 'u8']])]);
  instruction = 25;
}

type SetAndVerifyCollectionParams = {
  metadata: PublicKey;
  collectionAuthority: PublicKey;
  collectionMint: PublicKey;
  updateAuthority: PublicKey;
  collectionMetadata: PublicKey;
  collectionMasterEdition: PublicKey;
  collectionAuthorityRecord?: PublicKey;
};

export class SetAndVerifyCollectionCollection extends Transaction {
  constructor(options: TransactionCtorFields, params: SetAndVerifyCollectionParams) {
    super(options);
    const { feePayer } = options;
    const {
      metadata,
      collectionAuthority,
      collectionMint,
      updateAuthority,
      collectionMetadata,
      collectionMasterEdition,
      collectionAuthorityRecord,
    } = params;

    const data = SetAndVerifyCollectionArgs.serialize();
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
        pubkey: updateAuthority,
        isSigner: false,
        isWritable: false,
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
