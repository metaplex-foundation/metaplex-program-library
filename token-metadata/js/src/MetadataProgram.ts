import { PublicKey } from '@solana/web3.js';
import { Program, config } from '@metaplex-foundation/mpl-core';

export class MetadataProgram extends Program {
  static readonly PREFIX = 'metadata';
  static readonly EDITION = 'edition';
  static readonly USER = 'user';
  static readonly COLLECTION_AUTHORITY = 'collection_authority';
  static readonly BURN = 'burn';
  static readonly PUBKEY = new PublicKey(config.programs.metadata);

  static async findEditionAccount(
    mint: PublicKey,
    editionNumber: string,
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(MetadataProgram.PREFIX, 'utf8'),
        MetadataProgram.PUBKEY.toBuffer(),
        mint.toBuffer(),
        Buffer.from(MetadataProgram.EDITION, 'utf8'),
        Buffer.from(editionNumber, 'utf8'),
      ],
      MetadataProgram.PUBKEY,
    );
  }
  static async findMasterEditionAccount(mint: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(MetadataProgram.PREFIX, 'utf8'),
        MetadataProgram.PUBKEY.toBuffer(),
        mint.toBuffer(),
        Buffer.from(MetadataProgram.EDITION, 'utf8'),
      ],
      MetadataProgram.PUBKEY,
    );
  }

  static async findMetadataAccount(mint: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(MetadataProgram.PREFIX, 'utf8'),
        MetadataProgram.PUBKEY.toBuffer(),
        mint.toBuffer(),
      ],
      MetadataProgram.PUBKEY,
    );
  }

  static async findUseAuthorityAccount(
    mint: PublicKey,
    authority: PublicKey,
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(MetadataProgram.PREFIX, 'utf8'),
        MetadataProgram.PUBKEY.toBuffer(),
        mint.toBuffer(),
        Buffer.from(MetadataProgram.USER, 'utf8'),
        authority.toBuffer(),
      ],
      MetadataProgram.PUBKEY,
    );
  }

  static async findCollectionAuthorityAccount(
    mint: PublicKey,
    authority: PublicKey,
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(MetadataProgram.PREFIX, 'utf8'),
        MetadataProgram.PUBKEY.toBuffer(),
        mint.toBuffer(),
        Buffer.from(MetadataProgram.COLLECTION_AUTHORITY, 'utf8'),
        authority.toBuffer(),
      ],
      MetadataProgram.PUBKEY,
    );
  }

  static async findProgramAsBurnerAccount(): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(MetadataProgram.PREFIX, 'utf8'),
        MetadataProgram.PUBKEY.toBuffer(),
        Buffer.from(MetadataProgram.BURN, 'utf8'),
      ],
      MetadataProgram.PUBKEY,
    );
  }
}
