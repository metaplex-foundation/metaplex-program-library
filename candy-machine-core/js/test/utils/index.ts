export * from './asserts';
export * from './constants';
export * from './errors';
export * from './minter';

import { Keypair, PublicKey } from '@solana/web3.js';
import { BN } from 'bn.js';
import { CandyMachineData } from '../../src/generated';
import { HIDDEN_SECTION } from './constants';

export async function getCandyMachinePDA(programId: PublicKey, base: Keypair): Promise<PublicKey> {
  return await PublicKey.findProgramAddress(
    [Buffer.from('candy_machine'), base.publicKey.toBuffer()],
    programId,
  ).then((result) => {
    return result[0];
  });
}

export function getCandyMachineSpace(data: CandyMachineData): number {
  if (data.hiddenSettings != null) {
    return HIDDEN_SECTION;
  } else {
    const items = new BN(data.itemsAvailable).toNumber();
    if (data.configLineSettings != null) {
      return (
        HIDDEN_SECTION +
        4 +
        items * (data.configLineSettings.nameLength + data.configLineSettings.uriLength) +
        (Math.floor(items / 8) + 1) +
        items * 4
      );
    } else {
      // this will only be used by the test that tries to create a candy machine without
      // either config line or hidden settings
      return HIDDEN_SECTION + 4 + (Math.floor(items / 8) + 1) + items * 4;
    }
  }
}
