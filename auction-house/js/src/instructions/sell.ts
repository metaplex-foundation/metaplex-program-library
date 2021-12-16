import { AccountMeta, PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type SellInstructionArgs = {
  tradeStateBump: number;
  freeTradeStateBump: number;
  programAsSignerBump: number;
  buyerPrice: beet.bignum;
  tokenSize: beet.bignum;
};
const sellInstructionArgsStruct = new beet.BeetArgsStruct<SellInstructionArgs>([
  ['tradeStateBump', beet.u8],
  ['freeTradeStateBump', beet.u8],
  ['programAsSignerBump', beet.u8],
  ['buyerPrice', beet.u64],
  ['tokenSize', beet.u64],
]);
export type SellInstructionAccounts = {
  wallet: PublicKey;
  tokenAccount: PublicKey;
  metadata: PublicKey;
  authority: PublicKey;
  auctionHouse: PublicKey;
  auctionHouseFeeAccount: PublicKey;
  sellerTradeState: PublicKey;
  freeSellerTradeState: PublicKey;
  tokenProgram: PublicKey;
  systemProgram: PublicKey;
  programAsSigner: PublicKey;
  rent: PublicKey;
};

export function createSellInstruction(
  accounts: SellInstructionAccounts,
  args: SellInstructionArgs,
) {
  const {
    wallet,
    tokenAccount,
    metadata,
    authority,
    auctionHouse,
    auctionHouseFeeAccount,
    sellerTradeState,
    freeSellerTradeState,
    tokenProgram,
    systemProgram,
    programAsSigner,
    rent,
  } = accounts;

  const [data, _] = sellInstructionArgsStruct.serialize(args);
  const keys: AccountMeta[] = [
    {
      pubkey: wallet,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: tokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: metadata,
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
      pubkey: sellerTradeState,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: freeSellerTradeState,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: tokenProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: systemProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: programAsSigner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: rent,
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
