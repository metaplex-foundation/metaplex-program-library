import { AccountInfo, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import bs58 from 'bs58';
import { AnyPublicKey, StringPublicKey } from '@metaplex/types';
import { getBNFromData, TupleNumericType } from '@metaplex/utils';
import { Account } from '../../../Account';
import { MetaplexKey, MetaplexProgram } from '../MetaplexProgram';
import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '@metaplex/errors';
import { Buffer } from 'buffer';

export enum WinningConfigType {
  /// You may be selling your one-of-a-kind NFT for the first time, but not it's accompanying Metadata,
  /// of which you would like to retain ownership. You get 100% of the payment the first sale, then
  /// royalties forever after.
  ///
  /// You may be re-selling something like a Limited/Open Edition print from another auction,
  /// a master edition record token by itself (Without accompanying metadata/printing ownership), etc.
  /// This means artists will get royalty fees according to the top level royalty % on the metadata
  /// split according to their percentages of contribution.
  ///
  /// No metadata ownership is transferred in this instruction, which means while you may be transferring
  /// the token for a limited/open edition away, you would still be (nominally) the owner of the limited edition
  /// metadata, though it confers no rights or privileges of any kind.
  TokenOnlyTransfer,
  /// Means you are auctioning off the master edition record and it's metadata ownership as well as the
  /// token itself. The other person will be able to mint authorization tokens and make changes to the
  /// artwork.
  FullRightsTransfer,
  /// Means you are using authorization tokens to print off editions during the auction using
  /// from a MasterEditionV1
  PrintingV1,
  /// Means you are using the MasterEditionV2 to print off editions
  PrintingV2,
  /// Means you are using a MasterEditionV2 as a participation prize.
  Participation,
}

export enum WinningConstraint {
  NoParticipationPrize = 0,
  ParticipationPrizeGiven = 1,
}

export enum NonWinningConstraint {
  NoParticipationPrize = 0,
  GivenForFixedPrice = 1,
  GivenForBidPrice = 2,
}

export interface AmountRange {
  amount: BN;
  length: BN;
}

export interface ParticipationConfigV2 {
  winnerConstraint: WinningConstraint;
  nonWinningConstraint: NonWinningConstraint;
  fixedPrice: BN | null;
}

export interface ParticipationStateV2 {
  collectedToAcceptPayment: BN;
}

export interface SafetyDepositConfigData {
  key: MetaplexKey;
  auctionManager: StringPublicKey;
  order: BN;
  winningConfigType: WinningConfigType;
  amountType: TupleNumericType;
  lengthType: TupleNumericType;
  amountRanges: AmountRange[];
  participationConfig: ParticipationConfigV2 | null;
  participationState: ParticipationStateV2 | null;
}

export class SafetyDepositConfig extends Account<SafetyDepositConfigData> {
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);

    if (!this.assertOwner(MetaplexProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (!SafetyDepositConfig.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = deserialize(this.info.data);
  }

  static isCompatible(data: Buffer) {
    return data[0] === MetaplexKey.SafetyDepositConfigV1;
  }

  static async getPDA(auctionManager: AnyPublicKey, safetyDeposit: AnyPublicKey) {
    return MetaplexProgram.findProgramAddress([
      Buffer.from(MetaplexProgram.PREFIX),
      MetaplexProgram.PUBKEY.toBuffer(),
      new PublicKey(auctionManager).toBuffer(),
      new PublicKey(safetyDeposit).toBuffer(),
    ]);
  }
}

const deserialize = (buffer: Buffer) => {
  const data: SafetyDepositConfigData = {
    key: MetaplexKey.SafetyDepositConfigV1,
    auctionManager: bs58.encode(buffer.slice(1, 33)),
    order: new BN(buffer.slice(33, 41), 'le'),
    winningConfigType: buffer[41],
    amountType: buffer[42],
    lengthType: buffer[43],
    amountRanges: [],
    participationConfig: null,
    participationState: null,
  };

  const lengthOfArray = new BN(buffer.slice(44, 48), 'le');
  let offset = 48;

  for (let i = 0; i < lengthOfArray.toNumber(); i++) {
    const amount = getBNFromData(buffer, offset, data.amountType);
    offset += data.amountType;
    const length = getBNFromData(buffer, offset, data.lengthType);
    offset += data.lengthType;
    data.amountRanges.push({ amount, length });
  }

  if (buffer[offset] == 0) {
    offset += 1;
    data.participationConfig = null;
  } else {
    // pick up participation config manually
    const winnerConstraint = buffer[offset + 1];
    const nonWinningConstraint = buffer[offset + 2];
    let fixedPrice: BN | null = null;
    offset += 3;

    if (buffer[offset] == 1) {
      fixedPrice = new BN(buffer.slice(offset + 1, offset + 9), 'le');
      offset += 9;
    } else {
      offset += 1;
    }
    data.participationConfig = {
      winnerConstraint,
      nonWinningConstraint,
      fixedPrice,
    };
  }

  if (buffer[offset] == 0) {
    offset += 1;
    data.participationState = null;
  } else {
    // pick up participation state manually
    const collectedToAcceptPayment = new BN(buffer.slice(offset + 1, offset + 9), 'le');
    offset += 9;
    data.participationState = {
      collectedToAcceptPayment,
    };
  }

  return data;
};
