import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmRawTransaction,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
  Transaction,
} from '@solana/web3.js';
import {
  createInitializeCandyMachineInstruction,
  createMintNftInstruction,
  createSetLockupSettingsInstruction,
  LockupSettings,
  LockupType,
  PROGRAM_ID,
} from '../src/generated';
import test from 'tape';
import { LOCALHOST } from '@metaplex-foundation/amman-client';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MintLayout,
  Token,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { Edition, Metadata, MetadataProgram } from '@metaplex-foundation/mpl-token-metadata';
import { programs } from '@cardinal/token-manager';
import {
  CONFIG_ARRAY_START,
  CONFIG_LINE_SIZE,
  findLockupSettingsId,
  remainingAccountsForLockup,
} from '../src/utils';
import { amman } from './utils';
import { BN } from '@project-serum/anchor';

const walletKeypair = Keypair.generate();
const connection = new Connection(LOCALHOST, 'confirmed');
const candyMachineKeypair = Keypair.generate();
const nftToMintKeypair = Keypair.generate();
const ITEMS_AVAILABLE = 10;
let tokenAccountToReceive: PublicKey;

const uuidFromConfigPubkey = (configAccount: PublicKey) => {
  return configAccount.toBase58().slice(0, 6);
};

test('Candy machine initialize with lockup settings', async (t) => {
  await amman.airdrop(connection, walletKeypair.publicKey, 30);
  const [lockupSettingsId] = await findLockupSettingsId(candyMachineKeypair.publicKey);

  const initIx = createInitializeCandyMachineInstruction(
    {
      candyMachine: candyMachineKeypair.publicKey,
      wallet: walletKeypair.publicKey,
      authority: walletKeypair.publicKey,
      payer: walletKeypair.publicKey,
    },
    {
      data: {
        uuid: uuidFromConfigPubkey(candyMachineKeypair.publicKey),
        price: new BN(10),
        symbol: 'SYM',
        sellerFeeBasisPoints: 10,
        maxSupply: new BN(10),
        isMutable: true,
        retainAuthority: true,
        goLiveDate: new BN(Date.now() / 1000),
        endSettings: null,
        creators: [
          {
            address: candyMachineKeypair.publicKey,
            verified: true,
            share: 100,
          },
        ],
        hiddenSettings: null,
        whitelistMintSettings: null,
        itemsAvailable: new BN(ITEMS_AVAILABLE),
        gatekeeper: null,
      },
    },
  );

  const lockupInitIx = createSetLockupSettingsInstruction(
    {
      candyMachine: candyMachineKeypair.publicKey,
      authority: walletKeypair.publicKey,
      lockupSettings: lockupSettingsId,
      payer: walletKeypair.publicKey,
    },
    {
      lockupType: Number(LockupType.DurationSeconds),
      number: new BN(5),
    },
  );

  const tx = new Transaction();
  const size =
    CONFIG_ARRAY_START +
    4 +
    ITEMS_AVAILABLE * CONFIG_LINE_SIZE +
    8 +
    2 * (Math.floor(ITEMS_AVAILABLE / 8) + 1);
  const rent_exempt_lamports = await connection.getMinimumBalanceForRentExemption(size);
  tx.instructions = [
    SystemProgram.createAccount({
      fromPubkey: walletKeypair.publicKey,
      newAccountPubkey: candyMachineKeypair.publicKey,
      space: size,
      lamports: rent_exempt_lamports,
      programId: PROGRAM_ID,
    }),
    initIx,
    lockupInitIx,
  ];
  tx.feePayer = walletKeypair.publicKey;
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  tx.sign(walletKeypair, candyMachineKeypair);
  await sendAndConfirmRawTransaction(connection, tx.serialize());

  const lockupSettings = await LockupSettings.fromAccountAddress(connection, lockupSettingsId);
  t.assert(lockupSettings.lockupType === LockupType.DurationSeconds);
  t.assert(Number(lockupSettings.number) === 5);
});

test('Mint with lockup', async (t) => {
  tokenAccountToReceive = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    nftToMintKeypair.publicKey,
    walletKeypair.publicKey,
    false,
  );

  const metadataId = await Metadata.getPDA(nftToMintKeypair.publicKey);
  const masterEditionId = await Edition.getPDA(nftToMintKeypair.publicKey);
  const [tokenManagerId] = await programs.tokenManager.pda.findTokenManagerAddress(
    nftToMintKeypair.publicKey,
  );
  const [timeInvalidatorId] = await programs.timeInvalidator.pda.findTimeInvalidatorAddress(
    tokenManagerId,
  );
  const [candyMachineCreatorId, candyMachineCreatorIdBump] = await PublicKey.findProgramAddress(
    [Buffer.from('candy_machine'), candyMachineKeypair.publicKey.toBuffer()],
    PROGRAM_ID,
  );

  const mintIx = createMintNftInstruction(
    {
      candyMachine: candyMachineKeypair.publicKey,
      candyMachineCreator: candyMachineCreatorId,
      payer: walletKeypair.publicKey,
      wallet: walletKeypair.publicKey,
      metadata: metadataId,
      mint: nftToMintKeypair.publicKey,
      mintAuthority: walletKeypair.publicKey,
      updateAuthority: walletKeypair.publicKey,
      masterEdition: masterEditionId,
      tokenMetadataProgram: MetadataProgram.PUBKEY,
      clock: SYSVAR_CLOCK_PUBKEY,
      recentBlockhashes: SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
      instructionSysvarAccount: SYSVAR_INSTRUCTIONS_PUBKEY,
    },
    {
      creatorBump: candyMachineCreatorIdBump,
    },
  );

  const instructions = [
    SystemProgram.createAccount({
      fromPubkey: walletKeypair.publicKey,
      newAccountPubkey: nftToMintKeypair.publicKey,
      space: MintLayout.span,
      lamports: await connection.getMinimumBalanceForRentExemption(MintLayout.span),
      programId: TOKEN_PROGRAM_ID,
    }),
    Token.createInitMintInstruction(
      TOKEN_PROGRAM_ID,
      nftToMintKeypair.publicKey,
      0,
      walletKeypair.publicKey,
      walletKeypair.publicKey,
    ),
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      nftToMintKeypair.publicKey,
      tokenAccountToReceive,
      walletKeypair.publicKey,
      walletKeypair.publicKey,
    ),
    Token.createMintToInstruction(
      TOKEN_PROGRAM_ID,
      nftToMintKeypair.publicKey,
      tokenAccountToReceive,
      walletKeypair.publicKey,
      [],
      1,
    ),
    {
      ...mintIx,
      keys: [
        ...mintIx.keys,
        // remaining accounts for locking
        ...(await remainingAccountsForLockup(
          candyMachineKeypair.publicKey,
          nftToMintKeypair.publicKey,
          tokenAccountToReceive,
        )),
      ],
    },
  ];
  const tx = new Transaction();
  tx.instructions = instructions;
  tx.feePayer = walletKeypair.publicKey;
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  tx.sign(walletKeypair, nftToMintKeypair);
  await sendAndConfirmRawTransaction(connection, tx.serialize());

  const checkIssuerTokenAccount = await new Token(
    connection,
    nftToMintKeypair.publicKey,
    TOKEN_PROGRAM_ID,
    walletKeypair,
  ).getAccountInfo(tokenAccountToReceive);

  // assert is frozen
  t.assert(checkIssuerTokenAccount.isFrozen === true);
  // assert amount 1
  t.assert(checkIssuerTokenAccount.amount.toNumber() === 1);
  // assert time invalidator duration
  const timeInvalidator = await programs.timeInvalidator.accounts.getTimeInvalidator(
    connection,
    timeInvalidatorId,
  );
  t.assert(timeInvalidator.parsed.durationSeconds.toNumber() === 5);
});

test('Unlock fail', async (t) => {
  const [tokenManagerId] = await programs.tokenManager.pda.findTokenManagerAddress(
    nftToMintKeypair.publicKey,
  );
  const tokenManagerTokenAccount = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    nftToMintKeypair.publicKey,
    tokenManagerId,
    true,
  );
  const tokenManager = await programs.tokenManager.accounts.getTokenManager(
    connection,
    tokenManagerId,
  );

  try {
    const tx = new Transaction();
    const remainigAccountsForReturn = await programs.tokenManager.withRemainingAccountsForReturn(
      tx,
      connection,
      walletKeypair,
      tokenManager,
      true,
    );
    const invalidateIx = await programs.timeInvalidator.instruction.invalidate(
      connection,
      walletKeypair,
      nftToMintKeypair.publicKey,
      tokenManagerId,
      programs.tokenManager.TokenManagerKind.Edition,
      programs.tokenManager.TokenManagerState.Claimed,
      tokenManagerTokenAccount,
      tokenAccountToReceive,
      remainigAccountsForReturn,
    );
    tx.instructions = [invalidateIx];
    tx.feePayer = walletKeypair.publicKey;
    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
    tx.sign(walletKeypair);
    await sendAndConfirmRawTransaction(connection, tx.serialize());
  } catch (e) {
    const checkIssuerTokenAccount = await new Token(
      connection,
      nftToMintKeypair.publicKey,
      TOKEN_PROGRAM_ID,
      walletKeypair,
    ).getAccountInfo(tokenAccountToReceive);

    // assert is frozen
    t.assert(checkIssuerTokenAccount.isFrozen === true);
    // assert amount 1
    t.assert(checkIssuerTokenAccount.amount.toNumber() === 1);
    return;
  }
  t.assert(false);
});

test('Unlock after 5 seconds', async (t) => {
  // wait 6 seconds (duration)
  await new Promise((r) => setTimeout(r, 6000));

  const [tokenManagerId] = await programs.tokenManager.pda.findTokenManagerAddress(
    nftToMintKeypair.publicKey,
  );
  const tokenManagerTokenAccount = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    nftToMintKeypair.publicKey,
    tokenManagerId,
    true,
  );
  const tokenManager = await programs.tokenManager.accounts.getTokenManager(
    connection,
    tokenManagerId,
  );

  const tx = new Transaction();
  const remainigAccountsForReturn = await programs.tokenManager.withRemainingAccountsForReturn(
    tx,
    connection,
    walletKeypair,
    tokenManager,
    true,
  );
  const invalidateIx = await programs.timeInvalidator.instruction.invalidate(
    connection,
    walletKeypair,
    nftToMintKeypair.publicKey,
    tokenManagerId,
    programs.tokenManager.TokenManagerKind.Edition,
    programs.tokenManager.TokenManagerState.Claimed,
    tokenManagerTokenAccount,
    tokenAccountToReceive,
    remainigAccountsForReturn,
  );
  tx.instructions = [invalidateIx];
  tx.feePayer = walletKeypair.publicKey;
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  tx.sign(walletKeypair);
  await sendAndConfirmRawTransaction(connection, tx.serialize());

  const checkIssuerTokenAccount = await new Token(
    connection,
    nftToMintKeypair.publicKey,
    TOKEN_PROGRAM_ID,
    walletKeypair,
  ).getAccountInfo(tokenAccountToReceive);

  // assert is frozen
  t.assert(checkIssuerTokenAccount.isFrozen === false);
  // assert amount 1
  t.assert(checkIssuerTokenAccount.amount.toNumber() === 1);
  // assert token manager is gone
  try {
    await programs.tokenManager.accounts.getTokenManager(connection, tokenManagerId);
  } catch (e) {
    return;
  }
  t.assert(false);
});
