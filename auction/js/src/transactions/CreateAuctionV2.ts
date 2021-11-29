import { StringPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
import { AuctionProgram } from '../AuctionProgram';
import { Transaction } from '../../../Transaction';
import { PriceFloor } from '../accounts/Auction';
import { Args as CreateAuctionArgsType, CreateAuctionArgs, WinnerLimit } from './CreateAuction';

type Args = CreateAuctionArgsType & {
  instantSalePrice: BN | null;
  name: number[] | null;
};

export class CreateAuctionV2Args extends Borsh.Data<Args> {
  static readonly SCHEMA = new Map([
    ...CreateAuctionArgs.SCHEMA,
    ...this.struct([
      ['instantSalePrice', { kind: 'option', type: 'u64' }],
      ['name', { kind: 'option', type: [32] }],
    ]),
  ]);

  instruction = 7;
  /// How many winners are allowed for this auction. See AuctionData.
  winners: WinnerLimit;
  /// End time is the cut-off point that the auction is forced to end by. See AuctionData.
  endAuctionAt: BN | null;
  /// Gap time is how much time after the previous bid where the auction ends. See AuctionData.
  auctionGap: BN | null;
  /// Token mint for the SPL token used for bidding.
  tokenMint: StringPublicKey;
  /// Authority
  authority: StringPublicKey;
  /// The resource being auctioned. See AuctionData.
  resource: StringPublicKey;
  /// Set a price floor.
  priceFloor: PriceFloor;
  /// Add a tick size increment
  tickSize: BN | null;
  /// Add a minimum percentage increase each bid must meet.
  gapTickSizePercentage: number | null;
  /// Add a instant sale price.
  instantSalePrice: BN | null;
  /// Auction name
  name: number[] | null;
}

type CreateAuctionV2Params = {
  auction: PublicKey;
  auctionExtended: PublicKey;
  creator: PublicKey;
  args: CreateAuctionV2Args;
};

export class CreateAuctionV2 extends Transaction {
  constructor(options: TransactionCtorFields, params: CreateAuctionV2Params) {
    super(options);
    const { args, auction, auctionExtended, creator } = params;

    const data = CreateAuctionV2Args.serialize(args);

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: creator,
            isSigner: true,
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
            pubkey: SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SystemProgram.programId,
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
