import { PublicKey } from '@solana/web3.js';

export const TOKEN_PROGRAM_ID = new PublicKey(
  'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
);

export const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);

export const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
);

export const PRIVATE_METADATA_PROGRAM_ID = new PublicKey(
  '8SyzzxuZnMryDgLz6tWH3ubcEVikaVP3upq6cJce9jrL',
);

export const CURVE_DALEK_ONCHAIN_PROGRAM_ID = new PublicKey(
  '5wsqk1gtzFewn4Rx3w9uyYBYUPZJPr2uLTNJ7BGFav5g',
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

export async function getPrivateMetadata(
  mint: PublicKey,
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        mint.toBuffer(),
      ],
      PRIVATE_METADATA_PROGRAM_ID,
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
      PRIVATE_METADATA_PROGRAM_ID,
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
      PRIVATE_METADATA_PROGRAM_ID,
    )
  )[0];
};
