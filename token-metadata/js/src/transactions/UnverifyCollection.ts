import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { MetadataProgram } from '../MetadataProgram';

export class UnVerifyCollectionArgs extends Borsh.Data {
  static readonly SCHEMA = new Map([...UnVerifyCollectionArgs.struct([['instruction', 'u8']])]);
  instruction = 22;
}

type UnVerifyCollectionParams = {
  metadata: PublicKey;
  collectionUseAuthorityRecord?: PublicKey;
  collectionAuthority: PublicKey;
  collectionMint: PublicKey;
  collectionMetadata: PublicKey;
  collectionMasterEdition: PublicKey;
};

export class UnVerifyCollection extends Transaction {
  constructor(options: TransactionCtorFields, params: UnVerifyCollectionParams) {
    super(options);
    const { feePayer } = options;
    const {
      metadata,
      collectionAuthority,
      collectionMint,
      collectionMetadata,
      collectionMasterEdition,
      collectionUseAuthorityRecord,
    } = params;

    const data = UnVerifyCollectionArgs.serialize();
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
    if (collectionUseAuthorityRecord) {
      accounts.push({
        pubkey: collectionUseAuthorityRecord,
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
