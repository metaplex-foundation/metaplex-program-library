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
  CollectionPDA,
  createInitializeCandyMachineInstruction,
  createMintNftInstruction,
  createSetCollectionDuringMintInstruction,
  createSetCollectionInstruction,
  createSetLockupSettingsInstruction,
  LockupSettings,
  LockupType,
  PROGRAM_ID,
  WhitelistMintMode,
} from '../src/generated';
import test from 'tape';
import { LOCALHOST } from '@metaplex-foundation/amman-client';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MintLayout,
  Token,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import {
  CreateMasterEditionV3,
  CreateMetadataV2,
  Creator,
  DataV2,
  Edition,
  MasterEdition,
  Metadata,
  MetadataProgram,
} from '@metaplex-foundation/mpl-token-metadata';
import { programs } from '@cardinal/token-manager';
import {
  CONFIG_ARRAY_START,
  CONFIG_LINE_SIZE,
  findLockupSettingsId,
  remainingAccountsForLockup,
} from '../src/utils';
import { amman, createMintTransaction } from './utils';
import { BN } from '@project-serum/anchor';

const walletKeypair = Keypair.generate();
const connection = new Connection(LOCALHOST, 'confirmed');
const candyMachineKeypair = Keypair.generate();
const collectionMintKeypair = Keypair.generate();
const nftToMintKeypair = Keypair.generate();
const ITEMS_AVAILABLE = 10;
let tokenAccountToReceive: PublicKey;

const whitelistMintKeypair = Keypair.generate();
let whitelistMintTokenAccount: PublicKey;

const uuidFromConfigPubkey = (configAccount: PublicKey) => {
  return configAccount.toBase58().slice(0, 6);
};

test('Candy machine initialize with lockup settings and mint with whitelist token', async (t) => {
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
        goLiveDate: new BN(Date.now() / 1000 + 60000), // 60 seconds
        endSettings: null,
        creators: [
          {
            address: candyMachineKeypair.publicKey,
            verified: true,
            share: 100,
          },
        ],
        hiddenSettings: null,
        whitelistMintSettings: {
          mode: WhitelistMintMode.BurnEveryTime,
          mint: whitelistMintKeypair.publicKey,
          presale: true,
          discountPrice: 1,
        },
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
  tx.sign(walletKeypair, candyMachineKeypair, candyMachineKeypair);
  await sendAndConfirmRawTransaction(connection, tx.serialize());

  const lockupSettings = await LockupSettings.fromAccountAddress(connection, lockupSettingsId);
  t.assert(lockupSettings.lockupType === LockupType.DurationSeconds);
  t.assert(Number(lockupSettings.number) === 5);
});

test('Candy machine set collections', async (t) => {
  //// Collections
  const [collectionPdaId, _collectionPdaBump] = await PublicKey.findProgramAddress(
    [Buffer.from('collection'), candyMachineKeypair.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  const createCollectionMintTx = new Transaction();
  await createMintTransaction(
    createCollectionMintTx,
    connection,
    {
      signTransaction: async (tx) => tx,
      signAllTransactions: async (tx) => tx,
      publicKey: walletKeypair.publicKey,
    },
    walletKeypair.publicKey,
    collectionMintKeypair.publicKey,
    1,
  );

  const collectionMintMetadataId = await Metadata.getPDA(collectionMintKeypair.publicKey);
  const collectionMetadataTx = new CreateMetadataV2(
    { feePayer: walletKeypair.publicKey },
    {
      metadata: collectionMintMetadataId,
      metadataData: new DataV2({
        name: 'COLLECTION',
        symbol: 'COLL',
        uri: '',
        sellerFeeBasisPoints: 10,
        creators: [
          new Creator({
            address: walletKeypair.publicKey.toString(),
            verified: true,
            share: 100,
          }),
        ],
        collection: null,
        uses: null,
      }),
      updateAuthority: walletKeypair.publicKey,
      mint: collectionMintKeypair.publicKey,
      mintAuthority: walletKeypair.publicKey,
    },
  );

  const collectionMasterEditionId = await MasterEdition.getPDA(collectionMintKeypair.publicKey);
  const masterEditionTx = new CreateMasterEditionV3(
    {
      feePayer: walletKeypair.publicKey,
      recentBlockhash: (await connection.getRecentBlockhash('max')).blockhash,
    },
    {
      edition: collectionMasterEditionId,
      metadata: collectionMintMetadataId,
      updateAuthority: walletKeypair.publicKey,
      mint: collectionMintKeypair.publicKey,
      mintAuthority: walletKeypair.publicKey,
      maxSupply: new BN(0),
    },
  );

  const [collectionAuthorityRecordId] = await PublicKey.findProgramAddress(
    [
      Buffer.from('metadata'),
      MetadataProgram.PUBKEY.toBuffer(),
      collectionMintKeypair.publicKey.toBuffer(),
      Buffer.from('collection_authority'),
      collectionPdaId.toBuffer(),
    ],
    MetadataProgram.PUBKEY,
  );

  const setCollectionIx = createSetCollectionInstruction({
    candyMachine: candyMachineKeypair.publicKey,
    authority: walletKeypair.publicKey,
    collectionPda: collectionPdaId,
    payer: walletKeypair.publicKey,
    metadata: collectionMintMetadataId,
    mint: collectionMintKeypair.publicKey,
    edition: collectionMasterEditionId,
    collectionAuthorityRecord: collectionAuthorityRecordId,
    tokenMetadataProgram: MetadataProgram.PUBKEY,
  });

  const tx = new Transaction();
  tx.instructions = [
    ...createCollectionMintTx.instructions,
    ...collectionMetadataTx.instructions,
    ...masterEditionTx.instructions,
    setCollectionIx,
  ];
  tx.feePayer = walletKeypair.publicKey;
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  tx.sign(walletKeypair, collectionMintKeypair);
  await sendAndConfirmRawTransaction(connection, tx.serialize());

  const collectionPdaData = await CollectionPDA.fromAccountAddress(connection, collectionPdaId);
  t.assert(collectionPdaData.candyMachine.toString() === candyMachineKeypair.publicKey.toString());
});

test('Mint whitelist token', async (t) => {
  whitelistMintTokenAccount = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    whitelistMintKeypair.publicKey,
    walletKeypair.publicKey,
    false,
  );
  const instructions = [
    SystemProgram.createAccount({
      fromPubkey: walletKeypair.publicKey,
      newAccountPubkey: whitelistMintKeypair.publicKey,
      space: MintLayout.span,
      lamports: await connection.getMinimumBalanceForRentExemption(MintLayout.span),
      programId: TOKEN_PROGRAM_ID,
    }),
    Token.createInitMintInstruction(
      TOKEN_PROGRAM_ID,
      whitelistMintKeypair.publicKey,
      0,
      walletKeypair.publicKey,
      walletKeypair.publicKey,
    ),
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      whitelistMintKeypair.publicKey,
      whitelistMintTokenAccount,
      walletKeypair.publicKey,
      walletKeypair.publicKey,
    ),
    Token.createMintToInstruction(
      TOKEN_PROGRAM_ID,
      whitelistMintKeypair.publicKey,
      whitelistMintTokenAccount,
      walletKeypair.publicKey,
      [],
      1,
    ),
  ];
  const tx = new Transaction();
  tx.instructions = instructions;
  tx.feePayer = walletKeypair.publicKey;
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  tx.sign(walletKeypair, whitelistMintKeypair);
  await sendAndConfirmRawTransaction(connection, tx.serialize());

  const checkWhitelistTokenAccount = await new Token(
    connection,
    whitelistMintKeypair.publicKey,
    TOKEN_PROGRAM_ID,
    walletKeypair,
  ).getAccountInfo(whitelistMintTokenAccount);

  // assert is frozen
  t.assert(checkWhitelistTokenAccount.isFrozen === false);
  // assert amount 1
  t.assert(checkWhitelistTokenAccount.amount.toNumber() === 1);
});

test('Mint with lockup and whitelist', async (t) => {
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

  const [collectionPdaId, _collectionPdaBump] = await PublicKey.findProgramAddress(
    [Buffer.from('collection'), candyMachineKeypair.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  const collectionMintMetadataId = await Metadata.getPDA(collectionMintKeypair.publicKey);
  const collectionMasterEditionId = await MasterEdition.getPDA(collectionMintKeypair.publicKey);

  const [collectionAuthorityRecordId] = await PublicKey.findProgramAddress(
    [
      Buffer.from('metadata'),
      MetadataProgram.PUBKEY.toBuffer(),
      collectionMintKeypair.publicKey.toBuffer(),
      Buffer.from('collection_authority'),
      collectionPdaId.toBuffer(),
    ],
    MetadataProgram.PUBKEY,
  );

  const setCollectionDuringMintIx = createSetCollectionDuringMintInstruction({
    candyMachine: candyMachineKeypair.publicKey,
    metadata: metadataId,
    payer: walletKeypair.publicKey,
    collectionPda: collectionPdaId,
    tokenMetadataProgram: MetadataProgram.PUBKEY,
    instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    collectionMint: collectionMintKeypair.publicKey,
    collectionMasterEdition: collectionMasterEditionId,
    collectionMetadata: collectionMintMetadataId,
    authority: walletKeypair.publicKey,
    collectionAuthorityRecord: collectionAuthorityRecordId,
  });

  const instructions = [
    {
      ...mintIx,
      keys: [
        ...mintIx.keys,
        // remaining accounts for whitelist
        {
          pubkey: whitelistMintTokenAccount,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: whitelistMintKeypair.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: walletKeypair.publicKey,
          isSigner: false,
          isWritable: false,
        },
        // remaining accounts for minting the token during execution
        {
          pubkey: tokenAccountToReceive,
          isSigner: false,
          isWritable: true,
        },
        // remaining accounts for locking
        ...(await remainingAccountsForLockup(
          candyMachineKeypair.publicKey,
          nftToMintKeypair.publicKey,
          tokenAccountToReceive,
        )),
      ],
    },
    setCollectionDuringMintIx,
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
  const tokenManager = await programs.tokenManager.accounts.getTokenManager(
    connection,
    tokenManagerId,
  );
  t.assert(timeInvalidator.parsed.durationSeconds.toNumber() === 5);
  t.assert(
    tokenManager.parsed.recipientTokenAccount.toString() ===
      checkIssuerTokenAccount.address.toString(),
  );
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
