import { Borsh } from '@metaplex/utils';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { Transaction } from '../../../Transaction';
import { MetadataProgram } from '../MetadataProgram';

export class UpdatePrimarySaleHappenedViaTokenArgs extends Borsh.Data {
  static readonly SCHEMA = this.struct([['instruction', 'u8']]);

  instruction = 4;
}

type UpdatePrimarySaleHappenedViaTokenParams = {
  metadata: PublicKey;
  owner: PublicKey;
  tokenAccount: PublicKey;
};

export class UpdatePrimarySaleHappenedViaToken extends Transaction {
  constructor(options: TransactionCtorFields, params: UpdatePrimarySaleHappenedViaTokenParams) {
    super(options);
    const { metadata, owner, tokenAccount } = params;

    const data = UpdatePrimarySaleHappenedViaTokenArgs.serialize();

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: metadata,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: owner,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: tokenAccount,
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
