import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { MetadataProgram } from '../MetadataProgram';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

export class ThawDelegatedAccountArgs extends Borsh.Data {
  static readonly SCHEMA = new Map([...ThawDelegatedAccountArgs.struct([['instruction', 'u8']])]);
  instruction = 27;
}

type ThawDelegatedAccountParams = {
  delegate: PublicKey;
  token_account: PublicKey;
  edition: PublicKey;
  mint: PublicKey;
};

export class ThawDelegatedAccount extends Transaction {
  constructor(options: TransactionCtorFields, params: ThawDelegatedAccountParams) {
    super(options);
    const { delegate, token_account, edition, mint } = params;

    const data = ThawDelegatedAccountArgs.serialize();

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: delegate,
            isSigner: true,
            isWritable: true,
          },
          {
            pubkey: token_account,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: edition,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: mint,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: TOKEN_PROGRAM_ID,
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
