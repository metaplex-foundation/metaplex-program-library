import { PublicKey } from '@solana/web3.js';
import { MembershipTokenProgram } from 'src/MembershipToken';

const VAULT_OWNER_PREFIX = 'mt_vault';

export const findVaultOwnerAddress = (mint: PublicKey, store: PublicKey) => {
  return MembershipTokenProgram.findProgramAddress([
    Buffer.from(VAULT_OWNER_PREFIX),
    mint.toBuffer(),
    store.toBuffer(),
  ]);
};
