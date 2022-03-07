import { PublicKey } from '@solana/web3.js';
import { METADATA_PREFIX, METADATA_PROGRAM_ID } from './consts';

export async function pdaForMetadata(mint: PublicKey) {
  const [metadataPDA] = await PublicKey.findProgramAddress(
    [Buffer.from(METADATA_PREFIX), METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    METADATA_PROGRAM_ID,
  );
  return metadataPDA;
}
