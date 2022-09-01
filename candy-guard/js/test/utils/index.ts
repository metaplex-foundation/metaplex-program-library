export * from './asserts';
export * from './constants';
export * from './errors';

import { Keypair, PublicKey } from '@solana/web3.js';

export async function getCandyGuardPDA(programId: PublicKey, base: Keypair): Promise<PublicKey> {
  return await PublicKey.findProgramAddress(
    [Buffer.from('candy_guard'), base.publicKey.toBuffer()],
    programId,
  ).then((result) => {
    return result[0];
  });
}
