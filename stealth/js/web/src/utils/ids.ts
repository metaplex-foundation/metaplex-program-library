import {
  Keypair,
  PublicKey,
} from '@solana/web3.js';

import * as bs58 from 'bs58';

export const TOKEN_PROGRAM_ID = new PublicKey(
  'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
);

export const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);

export const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
);

export const STEALTH_PROGRAM_ID = new PublicKey(
  'privzjrXhtea8kKt3uE94X34AHaiLj2Vbwd51y3aUSi',
);

export const CURVE_DALEK_ONCHAIN_PROGRAM_ID = new PublicKey(
  'curveSS6UodDcBHTgerBXQxzW43kctcPe1dwT7yWaox',
);

export async function getMetadata(
  mint: PublicKey,
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID,
    )
  )[0];
};

export async function getStealth(
  mint: PublicKey,
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        mint.toBuffer(),
      ],
      STEALTH_PROGRAM_ID,
    )
  )[0];
};

export async function getElgamalPubkeyAddress(
  wallet: PublicKey,
  mint: PublicKey,
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        wallet.toBuffer(),
        mint.toBuffer(),
      ],
      STEALTH_PROGRAM_ID,
    )
  )[0];
};

export async function getTransferBufferAddress(
  wallet: PublicKey,
  mint: PublicKey,
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from('transfer'),
        wallet.toBuffer(),
        mint.toBuffer(),
      ],
      STEALTH_PROGRAM_ID,
    )
  )[0];
};

export const parseAddress = (address: string): PublicKey | null => {
  try {
    return new PublicKey(address);
  } catch {
    return null;
  }
};

export const parseKeypair = (secret: string): Keypair | null => {
  try {
    return Keypair.fromSecretKey(bs58.decode(secret));
  } catch {
    return null;
  }
};
