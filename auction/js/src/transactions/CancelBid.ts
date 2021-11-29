import { StringPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { Transaction } from '../../../Transaction';
import { AuctionProgram } from '../AuctionProgram';

export class CancelBidArgs extends Borsh.Data<{ resource: StringPublicKey }> {
  static readonly SCHEMA = this.struct([
    ['instruction', 'u8'],
    ['resource', 'pubkeyAsString'],
  ]);

  instruction = 0;
  resource: StringPublicKey;
}

type CancelBidParams = {
  auction: PublicKey;
  auctionExtended: PublicKey;
  bidderPot: PublicKey;
  bidderMeta: PublicKey;
  bidder: PublicKey;
  bidderToken: PublicKey;
  bidderPotToken: PublicKey;
  tokenMint: PublicKey;
  resource: PublicKey;
};

export class CancelBid extends Transaction {
  constructor(options: TransactionCtorFields, params: CancelBidParams) {
    super(options);
    const {
      auction,
      auctionExtended,
      bidderPot,
      bidderMeta,
      bidder,
      bidderToken,
      bidderPotToken,
      tokenMint,
      resource,
    } = params;

    const data = CancelBidArgs.serialize({ resource: resource.toString() });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: bidder,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: bidderToken,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: bidderPot,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: bidderPotToken,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: bidderMeta,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: auction,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: auctionExtended,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: tokenMint,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: SYSVAR_CLOCK_PUBKEY,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: TOKEN_PROGRAM_ID,
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
