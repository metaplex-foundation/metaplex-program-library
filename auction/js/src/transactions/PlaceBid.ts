import { Borsh } from '@metaplex/utils';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { AuctionProgram } from '../AuctionProgram';
import { Transaction } from '../../../Transaction';
import { StringPublicKey } from '@metaplex/types';
import BN from 'bn.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

export class PlaceBidArgs extends Borsh.Data<{ resource: StringPublicKey; amount: BN }> {
  static readonly SCHEMA = this.struct([
    ['instruction', 'u8'],
    ['amount', 'u64'],
    ['resource', 'pubkeyAsString'],
  ]);

  instruction = 6;
  resource: StringPublicKey;
  amount: BN;
}

type PlaceBidParams = {
  auction: PublicKey;
  auctionExtended: PublicKey;
  bidderPot: PublicKey;
  bidderMeta: PublicKey;
  bidder: PublicKey;
  bidderToken: PublicKey;
  bidderPotToken: PublicKey;
  tokenMint: PublicKey;
  transferAuthority: PublicKey;
  resource: PublicKey;
  amount: BN;
};

export class PlaceBid extends Transaction {
  constructor(options: TransactionCtorFields, params: PlaceBidParams) {
    super(options);
    const { feePayer } = options;
    const {
      auction,
      auctionExtended,
      bidderPot,
      bidderMeta,
      bidder,
      bidderToken,
      bidderPotToken,
      tokenMint,
      transferAuthority,
      resource,
      amount,
    } = params;

    const data = PlaceBidArgs.serialize({ resource: resource.toString(), amount });

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
            pubkey: transferAuthority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: feePayer,
            isSigner: true,
            isWritable: false,
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
