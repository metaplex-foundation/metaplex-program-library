import { Borsh } from '@metaplex/utils';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { Transaction } from '../../../Transaction';
import { MetadataProgram } from '../MetadataProgram';

export class SignMetadataArgs extends Borsh.Data {
  static readonly SCHEMA = this.struct([['instruction', 'u8']]);

  instruction = 7;
}

type SignMetadataParams = {
  metadata: PublicKey;
  creator: PublicKey;
};

export class SignMetadata extends Transaction {
  constructor(options: TransactionCtorFields, params: SignMetadataParams) {
    super(options);
    const { metadata, creator } = params;

    const data = SignMetadataArgs.serialize();

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: metadata,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: creator,
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
