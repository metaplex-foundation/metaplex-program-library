import { PublicKey } from '@solana/web3.js';
import { MetadataProgram } from '../MetadataProgram';

export async function programAsBurner(): Promise<[PublicKey, number]> {
  return PublicKey.findProgramAddress(
    [Buffer.from('metadata'), MetadataProgram.PUBKEY.toBuffer(), Buffer.from('burn')],
    MetadataProgram.PUBKEY,
  );
}
