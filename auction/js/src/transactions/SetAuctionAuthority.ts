import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { AuctionProgram } from '../AuctionProgram';

export class SetAuctionAuthorityArgs extends Borsh.Data {
  static readonly SCHEMA = SetAuctionAuthorityArgs.struct([['instruction', 'u8']]);

  instruction = 5;
}

type SetAuctionAuthorityParams = {
  auction: PublicKey;
  currentAuthority: PublicKey;
  newAuthority: PublicKey;
};

export class SetAuctionAuthority extends Transaction {
  constructor(options: TransactionCtorFields, params: SetAuctionAuthorityParams) {
    super(options);
    const { auction, currentAuthority, newAuthority } = params;

    const data = SetAuctionAuthorityArgs.serialize();

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: auction,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: currentAuthority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: newAuthority,
            isSigner: false,
            isWritable: false,
          },
        ],
        programId: AuctionProgram.PUBKEY,
        data,
      }),
    );
  }
}
