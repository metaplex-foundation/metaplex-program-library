import { AccountInfo, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '@metaplex/errors';
import { AnyPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { Account } from '../../../Account';
import { Edition } from './Edition';
import { MetadataKey, MetadataProgram } from '../MetadataProgram';
import { Buffer } from 'buffer';

type Args = { key: MetadataKey; ledger: number[] };
export class EditionMarkerData extends Borsh.Data<Args> {
  static readonly SCHEMA = this.struct([
    ['key', 'u8'],
    ['ledger', [31]],
  ]);

  key: MetadataKey;
  ledger: number[];

  constructor(args: Args) {
    super(args);
    this.key = MetadataKey.EditionMarker;
  }

  editionTaken(edition: number) {
    const editionOffset = edition % EditionMarker.DATA_SIZE;
    const indexOffset = Math.floor(editionOffset / 8);

    if (indexOffset > 30) {
      throw Error('Bad index for edition');
    }

    const positionInBitsetFromRight = 7 - (editionOffset % 8);
    const mask = Math.pow(2, positionInBitsetFromRight);
    const appliedMask = this.ledger[indexOffset] & mask;

    return appliedMask != 0;
  }
}

export class EditionMarker extends Account<EditionMarkerData> {
  static readonly DATA_SIZE = 248;

  constructor(key: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(key, info);

    if (!this.assertOwner(MetadataProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (!EditionMarker.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = EditionMarkerData.deserialize(this.info.data);
  }

  static async getPDA(mint: AnyPublicKey, edition: BN) {
    const editionNumber = Math.floor(edition.toNumber() / 248);

    return MetadataProgram.findProgramAddress([
      Buffer.from(MetadataProgram.PREFIX),
      MetadataProgram.PUBKEY.toBuffer(),
      new PublicKey(mint).toBuffer(),
      Buffer.from(Edition.EDITION_PREFIX),
      Buffer.from(editionNumber.toString()),
    ]);
  }

  static isCompatible(data: Buffer) {
    return data[0] === MetadataKey.EditionMarker;
  }
}
