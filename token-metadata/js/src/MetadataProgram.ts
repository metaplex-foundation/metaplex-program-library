import { PublicKey } from '@solana/web3.js';
import { Program, config } from '@metaplex-foundation/mpl-core';

export class MetadataProgram extends Program {
  static readonly PREFIX = 'metadata';
  static readonly EDITION = 'edition';
  static readonly USER = 'user';
  static readonly COLLECTION_AUTHORITY = 'collection_authority';
  static readonly BURN = 'burn';
  static readonly PUBKEY = new PublicKey(config.programs.metadata);

  static async find_edition_account(
    mint: PublicKey,
    edition_number: string,
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(MetadataProgram.PREFIX, 'utf8'),
        MetadataProgram.PUBKEY.toBuffer(),
        mint.toBuffer(),
        Buffer.from(MetadataProgram.EDITION, 'utf8'),
        Buffer.from(edition_number, 'utf8'),
      ],
      MetadataProgram.PUBKEY,
    );
  }
  static async find_master_edition_account(mint: PublicKey): Promise<[PublicKey, number]> {
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

  static async find_metadata_account(mint: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(MetadataProgram.PREFIX, 'utf8'),
        MetadataProgram.PUBKEY.toBuffer(),
        mint.toBuffer(),
      ],
      MetadataProgram.PUBKEY,
    );
  }

  static async find_use_authority_account(
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

  static async find_collection_authority_account(
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

  static async find_program_as_burner_account(): Promise<[PublicKey, number]> {
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
