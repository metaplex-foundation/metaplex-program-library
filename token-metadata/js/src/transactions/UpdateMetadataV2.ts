import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { DataV2 } from '../accounts/Metadata';
import { MetadataProgram } from '../MetadataProgram';

export class UpdateMetadataV2Args extends Borsh.Data<{
  data?: DataV2;
  updateAuthority?: string;
  primarySaleHappened: boolean | null;
  isMutable: boolean | null;
}> {
  static readonly SCHEMA = new Map([
    ...DataV2.SCHEMA,
    ...UpdateMetadataV2Args.struct([
      ['instruction', 'u8'],
      ['data', { kind: 'option', type: DataV2 }],
      ['updateAuthority', { kind: 'option', type: 'pubkeyAsString' }],
      ['primarySaleHappened', { kind: 'option', type: 'u8' }],
      ['isMutable', { kind: 'option', type: 'u8' }],
    ]),
  ]);

  instruction = 15;
  // NOTE: do not add "=null". This breaks serialization.
  data: DataV2 | null;
  updateAuthority: string | null;
  primarySaleHappened: boolean | null;
  isMutable: boolean | null;
}

type UpdateMetadataV2Params = {
  metadata: PublicKey;
  updateAuthority: PublicKey;
  metadataData?: DataV2;
  newUpdateAuthority?: PublicKey;
  primarySaleHappened?: boolean | null;
  isMutable?: boolean | null;
};

export class UpdateMetadataV2 extends Transaction {
  constructor(options: TransactionCtorFields, params: UpdateMetadataV2Params) {
    super(options);
    const {
      metadata,
      metadataData,
      updateAuthority,
      newUpdateAuthority,
      primarySaleHappened,
      isMutable,
    } = params;

    const data = UpdateMetadataV2Args.serialize({
      data: metadataData,
      updateAuthority: newUpdateAuthority && newUpdateAuthority.toString(),
      primarySaleHappened: primarySaleHappened || null,
      isMutable: isMutable || null,
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
            pubkey: updateAuthority,
            isSigner: true,
            isWritable: false,
          },
        ],
        programId: MetadataProgram.PUBKEY,
        data,
      }),
    );
  }
}
