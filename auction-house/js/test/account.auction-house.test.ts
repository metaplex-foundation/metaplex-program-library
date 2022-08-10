import { AccountInfo, Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import {
  AuctionHouse,
  AuctionHouseArgs,
  CreateAuctionHouseInstructionAccounts,
  CreateAuctionHouseInstructionArgs,
  createCreateAuctionHouseInstruction,
} from 'src/generated';
import test from 'tape';
import spok from 'spok';
import { Amman } from '@metaplex-foundation/amman-client';

const WRAPPED_SOL_MINT = new PublicKey('So11111111111111111111111111111111111111112');
export const AUCTION_HOUSE = 'auction_house';
export const FEE_PAYER = 'fee_payer';
export const TREASURY = 'treasury';

export const AUCTION_HOUSE_PROGRAM_ID = new PublicKey(
  'hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk',
);
const connectionURL = 'http://localhost:8899';
export const amman = Amman.instance({
  knownLabels: { ['hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk']: 'Auction House' },
  log: console.log,
});

function quickKeypair(): [PublicKey, Uint8Array] {
  const kp = Keypair.generate();
  return [kp.publicKey, kp.secretKey];
}

export const getAuctionHouse = async (
  creator: PublicKey,
  treasuryMint: PublicKey,
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [Buffer.from(AUCTION_HOUSE), creator.toBuffer(), treasuryMint.toBuffer()],
    AUCTION_HOUSE_PROGRAM_ID,
  );
};

export const getAuctionHouseFeeAcct = async (
  auctionHouse: PublicKey,
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [Buffer.from(AUCTION_HOUSE), auctionHouse.toBuffer(), Buffer.from(FEE_PAYER)],
    AUCTION_HOUSE_PROGRAM_ID,
  );
};

export const getAuctionHouseTreasuryAcct = async (
  auctionHouse: PublicKey,
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [Buffer.from(AUCTION_HOUSE), auctionHouse.toBuffer(), Buffer.from(TREASURY)],
    AUCTION_HOUSE_PROGRAM_ID,
  );
};

test('account auction-house: round trip serialization', async (t) => {
  const [creator] = quickKeypair();
  const [auctionHouseTreasury] = quickKeypair();
  const [treasuryWithdrawalDestination] = quickKeypair();
  const [feeWithdrawalDestination] = quickKeypair();
  const [treasuryMint] = quickKeypair();

  const args: AuctionHouseArgs = {
    auctionHouseFeeAccount: creator,
    auctionHouseTreasury,
    treasuryWithdrawalDestination,
    feeWithdrawalDestination,
    treasuryMint,
    authority: creator,
    creator,
    bump: 0,
    treasuryBump: 1,
    feePayerBump: 2,
    sellerFeeBasisPoints: 3,
    requiresSignOff: false,
    canChangeSalePrice: true,
    escrowPaymentBump: 255,
    hasAuctioneer: false,
    auctioneerAddress: PublicKey.default,
    scopes: Array(7).fill(false), // constant size field in the contract
  };

  const expected = AuctionHouse.fromArgs(args);
  const [data] = expected.serialize();

  const info: AccountInfo<Buffer> = {
    executable: false,
    data,
    owner: creator,
    lamports: 1000,
  };

  const actual = AuctionHouse.fromAccountInfo(info)[0];
  spok(t, actual, expected);
});

test('instruction auction-house: create auction-house', async (t) => {
  const authority = Keypair.generate();
  const treasuryWithdrawal = Keypair.generate();
  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = amman.payerTransactionHandler(connection, authority);
  await amman.airdrop(connection, authority.publicKey, 2);

  const [auctionHouse, ahBump] = await getAuctionHouse(authority.publicKey, WRAPPED_SOL_MINT);
  const [feeAccount, feeBump] = await getAuctionHouseFeeAcct(auctionHouse);
  const [treasuryAccount, treasuryBump] = await getAuctionHouseTreasuryAcct(auctionHouse);

  const accounts: CreateAuctionHouseInstructionAccounts = {
    treasuryMint: WRAPPED_SOL_MINT,
    payer: authority.publicKey,
    authority: authority.publicKey,
    feeWithdrawalDestination: authority.publicKey,
    treasuryWithdrawalDestination: treasuryWithdrawal.publicKey,
    treasuryWithdrawalDestinationOwner: treasuryWithdrawal.publicKey,
    auctionHouse: auctionHouse,
    auctionHouseFeeAccount: feeAccount,
    auctionHouseTreasury: treasuryAccount,
  };

  const args: CreateAuctionHouseInstructionArgs = {
    bump: ahBump,
    feePayerBump: feeBump,
    treasuryBump: treasuryBump,
    sellerFeeBasisPoints: 250,
    requiresSignOff: false,
    canChangeSalePrice: false,
  };
  const create_ah_instruction = createCreateAuctionHouseInstruction(accounts, args);
  const tx = new Transaction().add(create_ah_instruction);
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  const txId = await transactionHandler.sendAndConfirmTransaction(tx, [authority], {
    skipPreflight: true,
  });

  t.ok(txId);
});
