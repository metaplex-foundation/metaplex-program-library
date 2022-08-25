import {
  AccountInfo,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import { AuctionHouse, AuctionHouseArgs } from 'src/generated';
import test from 'tape';
import spok from 'spok';
import {
  getAuctionHouse,
  getAuctionHouseFeeAcct,
  getAuctionHouseTreasuryAcct,
  getAuctionHouseBuyerEscrow,
} from '../../cli/src/helpers/accounts';

import {
  CreateAuctionHouseInstructionAccounts,
  CreateAuctionHouseInstructionArgs,
  createCreateAuctionHouseInstruction,
  DepositInstructionAccounts,
  DepositInstructionArgs,
  createDepositInstruction,
  WithdrawInstructionAccounts,
  WithdrawInstructionArgs,
  createWithdrawInstruction,
} from 'src/generated';
import { Amman } from '@metaplex-foundation/amman-client';
import { LOCALHOST } from '@metaplex-foundation/amman';

const connectionURL = LOCALHOST;

const WRAPPED_SOL_MINT = new PublicKey('So11111111111111111111111111111111111111112');
const REQUIRED_RENT_EXEMPTION = 890_880;

const AUCTION_HOUSE_PROGRAM_ID = new PublicKey('hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk');
const amman = Amman.instance({
  knownLabels: { [AUCTION_HOUSE_PROGRAM_ID.toString()]: 'Auction House' },
  log: console.log,
});

function quickKeypair(): [PublicKey, Uint8Array] {
  const kp = Keypair.generate();
  return [kp.publicKey, kp.secretKey];
}

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

test('test auction-house instructions', async (t) => {
  const authority = Keypair.generate();
  const connection = new Connection(connectionURL, 'confirmed');

  test('instruction auction-house: create auction-house', async (t) => {
    const treasuryWithdrawal = Keypair.generate();
    const transactionHandler = amman.payerTransactionHandler(connection, authority);
    const authority_aidrop_sol = 2;
    await amman.airdrop(connection, authority.publicKey, authority_aidrop_sol);
    const authority_balance = await connection.getBalance(authority.publicKey);
    t.equal(authority_balance / LAMPORTS_PER_SOL, authority_aidrop_sol);

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
      requiresSignOff: true,
      canChangeSalePrice: false,
    };
    const create_ah_instruction = createCreateAuctionHouseInstruction(accounts, args);
    const tx = new Transaction().add(create_ah_instruction);
    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
    const txId = await transactionHandler.sendAndConfirmTransaction(tx, [authority], {
      skipPreflight: false,
    });
    t.ok(txId);
    t.end();
  });
  test('instruction auction-house: deposit and withdraw', async (t) => {
    const wallet = Keypair.generate();
    const transactionHandler = amman.payerTransactionHandler(connection, wallet);
    const [auctionHouse, {}] = await getAuctionHouse(authority.publicKey, WRAPPED_SOL_MINT);
    const [feeAccount, {}] = await getAuctionHouseFeeAcct(auctionHouse);
    const [escrowPaymentAccount, escrowPaymentBump] = await getAuctionHouseBuyerEscrow(
      auctionHouse,
      wallet.publicKey,
    );

    const auction_house_fee_account_pre_balance = await connection.getBalance(feeAccount);
    await amman.airdrop(connection, wallet.publicKey, 2);
    const wallet_sol_pre_balance = await connection.getBalance(wallet.publicKey);
    const deposit_amount = 1000;
    const expected_balance_post_withdraw = REQUIRED_RENT_EXEMPTION;

    const depositAccounts: DepositInstructionAccounts = {
      wallet: wallet.publicKey,
      paymentAccount: wallet.publicKey,
      transferAuthority: authority.publicKey,
      escrowPaymentAccount: escrowPaymentAccount,
      treasuryMint: WRAPPED_SOL_MINT,
      authority: authority.publicKey,
      auctionHouse: auctionHouse,
      auctionHouseFeeAccount: feeAccount,
    };

    const deposit_args: DepositInstructionArgs = {
      escrowPaymentBump: escrowPaymentBump,
      amount: deposit_amount,
    };

    const deposit_instruction = createDepositInstruction(depositAccounts, deposit_args);
    const deposit_tx = new Transaction().add(deposit_instruction);
    deposit_tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
    await transactionHandler.sendAndConfirmTransaction(deposit_tx, [wallet, authority], {
      skipPreflight: false,
    });

    const deposit_fee_paid = (await connection.getFeeForMessage(deposit_tx.compileMessage())).value;
    const wallet_sol_post_deposit_balance = await connection.getBalance(wallet.publicKey);
    const escrow_post_deposit_balance = await connection.getBalance(escrowPaymentAccount);

    t.equal(
      wallet_sol_post_deposit_balance,
      wallet_sol_pre_balance - deposit_amount - deposit_fee_paid - REQUIRED_RENT_EXEMPTION,
      'wallet_sol_post_deposit_balance',
    );
    t.equal(
      escrow_post_deposit_balance,
      deposit_amount + REQUIRED_RENT_EXEMPTION,
      'escrow_sol_post_deposit_balance',
    );

    // withdraw
    const withdrawAccounts: WithdrawInstructionAccounts = {
      wallet: wallet.publicKey,
      receiptAccount: wallet.publicKey,
      escrowPaymentAccount: escrowPaymentAccount,
      treasuryMint: WRAPPED_SOL_MINT,
      authority: authority.publicKey,
      auctionHouse: auctionHouse,
      auctionHouseFeeAccount: feeAccount,
    };

    const withdraw_args: WithdrawInstructionArgs = {
      escrowPaymentBump: escrowPaymentBump,
      amount: deposit_amount,
    };

    const withdraw_instruction = createWithdrawInstruction(withdrawAccounts, withdraw_args);
    const withdraw_tx = new Transaction().add(withdraw_instruction);
    withdraw_tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
    await transactionHandler.sendAndConfirmTransaction(withdraw_tx, [wallet, authority], {
      skipPreflight: false,
    });

    const withdraw_fee_paid = (await connection.getFeeForMessage(withdraw_tx.compileMessage()))
      .value;
    const escrow_post_withdraw_balance = await connection.getBalance(escrowPaymentAccount);
    const wallet_sol_post_withdraw_balance = await connection.getBalance(wallet.publicKey);

    t.equal(
      escrow_post_withdraw_balance,
      expected_balance_post_withdraw,
      'escrow balance post withdraw == expected',
    );
    t.equal(
      wallet_sol_post_withdraw_balance,
      wallet_sol_post_deposit_balance - withdraw_fee_paid + deposit_amount,
      'wallet balance post withdraw == expected',
    );

    const auction_house_fee_account_post_balance = await connection.getBalance(feeAccount);
    t.equal(
      auction_house_fee_account_pre_balance,
      auction_house_fee_account_post_balance,
      'auction_house_fee_account_pre_balance == auction_house_fee_account_post_balance',
    );
    t.end();
  });
  t.ok(true);
});
