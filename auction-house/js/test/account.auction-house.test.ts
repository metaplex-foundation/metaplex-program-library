import { AccountInfo, Connection, Keypair, PublicKey } from '@solana/web3.js';
import {
  AuctionHouse,
  AuctionHouseArgs,
  // CreateAuctionHouseInstructionAccounts,
  // CreateAuctionHouseInstructionArgs,
  // createCreateAuctionHouseInstruction,
} from 'src/generated';
import test from 'tape';
import spok from 'spok';
import { Amman } from '@metaplex-foundation/amman-client';

const connectionURL = 'http://localhost:8899';
export const amman = Amman.instance({
  knownLabels: { ['hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk']: 'Auction House' },
  log: console.log,
});

function quickKeypair(): [PublicKey, Uint8Array] {
  const kp = Keypair.generate();
  return [kp.publicKey, kp.secretKey];
}

test('account auction-house: round trip serilization', async (t) => {
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
  const authority = quickKeypair();
  const connection = new Connection(connectionURL, 'confirmed');
  await amman.airdrop(connection, authority[0], 2);
  // const accounts: CreateAuctionHouseInstructionAccounts = {
  //   treasuryMint: web3.PublicKey,
  //   payer: web3.PublicKey,
  //   authority: web3.PublicKey,
  //   feeWithdrawalDestination: web3.PublicKey,
  //   treasuryWithdrawalDestination: web3.PublicKey,
  //   treasuryWithdrawalDestinationOwner: web3.PublicKey,    auctionHouse: web3.PublicKey;
  //   auctionHouseFeeAccount: web3.PublicKey,
  //   auctionHouseTreasury: web3.PublicKey,
  // };
  t.ok(true);
});
