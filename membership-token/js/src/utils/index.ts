import { PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from '../consts';

const VAULT_OWNER_PREFIX = 'mt_vault';
const TREASURY_OWNER_PREFIX = 'holder';

export const findVaultOwnerAddress = (mint: PublicKey, store: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from(VAULT_OWNER_PREFIX), mint.toBuffer(), store.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};

export const findTresuryOwnerAddress = (treasuryMint: PublicKey, sellingResource: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from(TREASURY_OWNER_PREFIX), treasuryMint.toBuffer(), sellingResource.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};

export const checkByteSizes = (value: string, length: number): string => {
  const bytesLength = Buffer.from(value, 'utf8').byteLength;

  return bytesLength < length ? value + ' '.repeat(length - bytesLength) : value;
};
