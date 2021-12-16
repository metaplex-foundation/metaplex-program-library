import { AccountMeta, PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type CancelInstructionArgs = {
  buyerPrice: beet.bignum;
  tokenSize: beet.bignum;
};
const cancelInstructionArgsStruct = new beet.BeetArgsStruct<CancelInstructionArgs>([
  ['buyerPrice', beet.u64],
  ['tokenSize', beet.u64],
]);
export type CancelInstructionAccounts = {
  wallet: PublicKey;
  tokenAccount: PublicKey;
  tokenMint: PublicKey;
  authority: PublicKey;
  auctionHouse: PublicKey;
  auctionHouseFeeAccount: PublicKey;
  tradeState: PublicKey;
  tokenProgram: PublicKey;
};

export function createCancelInstruction(
  accounts: CancelInstructionAccounts,
  args: CancelInstructionArgs,
) {
  const {
    wallet,
    tokenAccount,
    tokenMint,
    authority,
    auctionHouse,
    auctionHouseFeeAccount,
    tradeState,
    tokenProgram,
  } = accounts;

  const [data, _] = cancelInstructionArgsStruct.serialize(args);
  const keys: AccountMeta[] = [
    {
      pubkey: wallet,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: tokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: tokenMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: authority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: auctionHouse,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: auctionHouseFeeAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: tradeState,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: tokenProgram,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new TransactionInstruction({
    programId: new PublicKey('hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk'),
    keys,
    data,
  });
  return ix;
}
