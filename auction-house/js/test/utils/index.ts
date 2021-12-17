import { clusterApiUrl } from '@solana/web3.js';
import { inspect } from 'util';
import debug from 'debug';
import test from 'tape';
import * as anchor from "@project-serum/anchor";
import {
  PublicKey,
} from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, Token } from '@solana/spl-token';
import { AccountLayout, u64 } from '@solana/spl-token';
import { LOCALHOST } from '@metaplex-foundation/amman';

export const logError = debug('mpl:tm-test:error');
export const logInfo = debug('mpl:tm-test:info');
export const logDebug = debug('mpl:tm-test:debug');
export const logTrace = debug('mpl:tm-test:trace');

export const programIds = {
  metadata: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
  vault: 'vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn',
  auction: 'auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8',
  metaplex: 'p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98',
  auctionhouse: '6Dtd5LSEZo3evRzWCjydocMJXtthFJKjcr8AnHdYrEuB',
};

// Address of the deployed program.
export const programId = new anchor.web3.PublicKey(programIds.auctionhouse);


export const DEVNET = clusterApiUrl('devnet');
export const connectionURL = process.env.USE_DEVNET != null ? DEVNET : LOCALHOST;
export const myWalletFile = process.env.ANCHOR_WALLET != null ? process.env.ANCHOR_WALLET : new String(process.env.HOME).concat("/.config/solana/id.json");

export function dump(x: object) {
  console.log(inspect(x, { depth: 5 }));
}

export function killStuckProcess() {
  // solana web socket keeps process alive for longer than necessary which we
  // "fix" here
  test.onFinish(() => process.exit(0));
}

export const AUCTION_HOUSE = "auction_house";
export const FEE_PAYER = 'fee_payer';
export const TREASURY = 'treasury';
export const SFBP = 1000;

export const WRAPPED_SOL_MINT = new PublicKey(
  'So11111111111111111111111111111111111111112',
);
export const TOKEN_PROGRAM_ID = new PublicKey(
  'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
);
export const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);

export const getAuctionHouse = async (
  creator: anchor.web3.PublicKey,
  treasuryMint: anchor.web3.PublicKey,
): Promise<[PublicKey, number]> => {
  return await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(AUCTION_HOUSE), creator.toBuffer(), treasuryMint.toBuffer()],
    programId,
  );
};

export const getAtaForMint = async (
  mint: anchor.web3.PublicKey,
  buyer: anchor.web3.PublicKey,
): Promise<[anchor.web3.PublicKey, number]> => {
  return await anchor.web3.PublicKey.findProgramAddress(
    [buyer.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
  );
};

export const getAuctionHouseFeeAcct = async (
  auctionHouse: anchor.web3.PublicKey,
): Promise<[PublicKey, number]> => {
  return await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from(AUCTION_HOUSE),
      auctionHouse.toBuffer(),
      Buffer.from(FEE_PAYER),
    ],
    programId,
  );
};

export const getAuctionHouseTreasuryAcct = async (
  auctionHouse: anchor.web3.PublicKey,
): Promise<[PublicKey, number]> => {
  return await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from(AUCTION_HOUSE),
      auctionHouse.toBuffer(),
      Buffer.from(TREASURY),
    ],
    programId,
  );
};

export const deserializeAccount = (data: Buffer) => {
  const accountInfo = AccountLayout.decode(data);
  accountInfo.mint = new PublicKey(accountInfo.mint);
  accountInfo.owner = new PublicKey(accountInfo.owner);
  accountInfo.amount = u64.fromBuffer(accountInfo.amount);

  if (accountInfo.delegateOption === 0) {
    accountInfo.delegate = null;
    accountInfo.delegatedAmount = new u64(0);
  } else {
    accountInfo.delegate = new PublicKey(accountInfo.delegate);
    accountInfo.delegatedAmount = u64.fromBuffer(accountInfo.delegatedAmount);
  }

  accountInfo.isInitialized = accountInfo.state !== 0;
  accountInfo.isFrozen = accountInfo.state === 2;

  if (accountInfo.isNativeOption === 1) {
    accountInfo.rentExemptReserve = u64.fromBuffer(accountInfo.isNative);
    accountInfo.isNative = true;
  } else {
    accountInfo.rentExemptReserve = null;
    accountInfo.isNative = false;
  }

  if (accountInfo.closeAuthorityOption === 0) {
    accountInfo.closeAuthority = null;
  } else {
    accountInfo.closeAuthority = new PublicKey(accountInfo.closeAuthority);
  }

  return accountInfo;
};

export const getPriceWithMantissa = async (
  price: number,
  mint: anchor.web3.PublicKey,
  walletKeyPair: any,
  anchorProgram: anchor.Program,
): Promise<number> => {
  const token = new Token(
    anchorProgram.provider.connection,
    new anchor.web3.PublicKey(mint),
    TOKEN_PROGRAM_ID,
    walletKeyPair,
  );

  const mintInfo = await token.getMintInfo();

  const mantissa = 10 ** mintInfo.decimals;

  return Math.ceil(price * mantissa);
};

export const getAuctionHouseBuyerEscrow = async (
  auctionHouse: anchor.web3.PublicKey,
  wallet: anchor.web3.PublicKey,
): Promise<[PublicKey, number]> => {
  return await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(AUCTION_HOUSE), auctionHouse.toBuffer(), wallet.toBuffer()],
    programId,
  );
};

export async function getTokenAmount(
  anchorProgram: anchor.Program,
  account: anchor.web3.PublicKey,
  mint: anchor.web3.PublicKey,
): Promise<number> {
  let amount = 0;
  if (!mint.equals(WRAPPED_SOL_MINT)) {
    try {
      const token =
        await anchorProgram.provider.connection.getTokenAccountBalance(account);
      amount = token.value.uiAmount * Math.pow(10, token.value.decimals);
    } catch (e) {
      console.log(e);
      console.log(
        'Account ',
        account.toBase58(),
        'didnt return value. Assuming 0 tokens.',
      );
    }
  } else {
    amount = await anchorProgram.provider.connection.getBalance(account);
  }
  return amount;
}
