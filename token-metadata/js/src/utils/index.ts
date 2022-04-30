import { bignum } from '@metaplex-foundation/beet';
import { PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from '../generated';

export const METADATA_PREFIX = 'metadata';
export const EDITION = 'edition';
export const COLLECTION_AUTHORITY = 'collection_authority';
export const USER = 'user';
export const BURN = 'burn';

export const findMetadataAddress = (mint: PublicKey): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [Buffer.from(METADATA_PREFIX), PROGRAM_ID.toBuffer(), mint.toBuffer()],
    PROGRAM_ID,
  );

export const findEditionAddress = (mint: PublicKey): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [Buffer.from(METADATA_PREFIX), PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from(EDITION)],
    PROGRAM_ID,
  );

export const findEditionMarkerAddress = (
  mint: PublicKey,
  edition: bignum,
): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [
      Buffer.from(METADATA_PREFIX),
      PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from(EDITION),
      Buffer.from(Math.floor(Number(edition) / 248).toString()),
    ],
    PROGRAM_ID,
  );

export const findCollectionAuthorityRecordAddress = (
  mint: PublicKey,
  newAuthority: PublicKey,
): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [
      Buffer.from(METADATA_PREFIX),
      PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from(COLLECTION_AUTHORITY),
      newAuthority.toBuffer(),
    ],
    PROGRAM_ID,
  );

export const findUseAuthorityAccount = (
  mint: PublicKey,
  authority: PublicKey,
): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [
      Buffer.from(METADATA_PREFIX),
      PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from(USER),
      authority.toBuffer(),
    ],
    PROGRAM_ID,
  );

export const findProgramAsBurnerAddress = (): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [Buffer.from(METADATA_PREFIX), PROGRAM_ID.toBuffer(), Buffer.from(BURN)],
    PROGRAM_ID,
  );
