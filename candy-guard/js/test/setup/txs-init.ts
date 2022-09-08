import {
  ConfirmedTransactionAssertablePromise,
  GenLabeledKeypair,
  LoadOrGenKeypair,
  LOCALHOST,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman-client';
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_SLOT_HASHES_PUBKEY,
  Transaction,
  TransactionInstruction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  AccountMeta,
} from '@solana/web3.js';
import { Test } from 'tape';
import { amman } from '.';
import {
  CANDY_MACHINE_PROGRAM,
  CandyMachineHelper,
  getCandyGuardPDA,
  METAPLEX_PROGRAM_ID,
} from '../utils';
import {
  CandyGuardData,
  createInitializeInstruction,
  createMintInstruction,
  createUpdateInstruction,
  createWrapInstruction,
  InitializeInstructionAccounts,
  InitializeInstructionArgs,
  MintInstructionAccounts,
  MintInstructionArgs,
  PROGRAM_ID,
  UpdateInstructionAccounts,
  UpdateInstructionArgs,
  WrapInstructionAccounts,
} from '../../src/generated';
import { CandyMachine } from '../../../../candy-core/js/src/generated';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  MintLayout,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';

const HELPER = new CandyMachineHelper();

export class InitTransactions {
  readonly getKeypair: LoadOrGenKeypair | GenLabeledKeypair;
  constructor(readonly resuseKeypairs = false) {
    this.getKeypair = resuseKeypairs ? amman.loadOrGenKeypair : amman.genLabeledKeypair;
  }

  async payer() {
    const [payer, payerPair] = await this.getKeypair('Payer');

    const connection = new Connection(LOCALHOST, 'confirmed');
    await amman.airdrop(connection, payer, 2);

    const transactionHandler = amman.payerTransactionHandler(connection, payerPair);

    return {
      fstTxHandler: transactionHandler,
      connection,
      payer,
      payerPair,
    };
  }

  async authority() {
    const [authority, authorityPair] = await this.getKeypair('Authority');

    const connection = new Connection(LOCALHOST, 'confirmed');
    await amman.airdrop(connection, authority, 2);

    const transactionHandler = amman.payerTransactionHandler(connection, authorityPair);

    return {
      fstTxHandler: transactionHandler,
      connection,
      authority,
      authorityPair,
    };
  }

  async minter() {
    const [minter, minterPair] = await this.getKeypair('Minter');

    const connection = new Connection(LOCALHOST, 'confirmed');
    await amman.airdrop(connection, minter, 2);

    const transactionHandler = amman.payerTransactionHandler(connection, minterPair);

    return {
      fstTxHandler: transactionHandler,
      connection,
      minter,
      minterPair,
    };
  }

  async initialize(
    t: Test,
    data: CandyGuardData,
    payer: Keypair,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; candyGuard: PublicKey }> {
    const [, keypair] = await this.getKeypair('Candy Guard Base Pubkey');
    const pda = await getCandyGuardPDA(PROGRAM_ID, keypair);
    amman.addr.addLabel('Candy Guard Account', pda);

    const accounts: InitializeInstructionAccounts = {
      candyGuard: pda,
      base: keypair.publicKey,
      authority: payer.publicKey,
      payer: payer.publicKey,
      systemProgram: SystemProgram.programId,
    };

    const args: InitializeInstructionArgs = {
      data: data,
    };

    const tx = new Transaction().add(createInitializeInstruction(accounts, args));

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer, keypair], 'tx: Initialize'),
      candyGuard: pda,
    };
  }

  async wrap(
    t: Test,
    candyGuard: PublicKey,
    candyMachine: PublicKey,
    payer: Keypair,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const accounts: WrapInstructionAccounts = {
      candyGuard,
      authority: payer.publicKey,
      candyMachine: candyMachine,
      candyMachineProgram: CANDY_MACHINE_PROGRAM,
    };

    const tx = new Transaction().add(createWrapInstruction(accounts));

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Wrap'),
    };
  }

  async update(
    t: Test,
    candyGuard: PublicKey,
    data: CandyGuardData,
    payer: Keypair,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const accounts: UpdateInstructionAccounts = {
      candyGuard,
      authority: payer.publicKey,
      payer: payer.publicKey,
      systemProgram: SystemProgram.programId,
    };

    const args: UpdateInstructionArgs = {
      data,
    };

    const tx = new Transaction().add(createUpdateInstruction(accounts, args));

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Update'),
    };
  }

  async mint(
    t: Test,
    candyGuard: PublicKey,
    candyMachine: PublicKey,
    payer: Keypair,
    mint: Keypair,
    handler: PayerTransactionHandler,
    connection: Connection,
    collection?: PublicKey | null,
    remainingAccounts?: AccountMeta[] | null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    // candy machine object
    const candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);

    // PDAs required for the mint

    // creator address
    const [candyMachineCreator, bump] = await PublicKey.findProgramAddress(
      [Buffer.from('candy_machine'), candyMachine.toBuffer()],
      CANDY_MACHINE_PROGRAM,
    );
    amman.addr.addLabel('Mint Creator', candyMachineCreator);

    // associated token address
    const [associatedToken] = await PublicKey.findProgramAddress(
      [payer.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    amman.addr.addLabel('Mint Associated Token', associatedToken);

    // metadata address
    const [metadataAddress] = await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), METAPLEX_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Mint Metadata', metadataAddress);

    // master edition address
    const [masterEdition] = await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        METAPLEX_PROGRAM_ID.toBuffer(),
        mint.publicKey.toBuffer(),
        Buffer.from('edition'),
      ],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Mint Master Edition', masterEdition);

    const collectionMint = candyMachineObject.collectionMint;

    const [collectionAuthorityRecord] = await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        METAPLEX_PROGRAM_ID.toBuffer(),
        collectionMint.toBuffer(),
        Buffer.from('collection_authority'),
        candyMachineObject.updateAuthority.toBuffer(),
      ],
      METAPLEX_PROGRAM_ID,
    );

    const [collectionMetadata] = await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), METAPLEX_PROGRAM_ID.toBuffer(), collectionMint.toBuffer()],
      METAPLEX_PROGRAM_ID,
    );
    const [collectionMasterEdition] = await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        METAPLEX_PROGRAM_ID.toBuffer(),
        collectionMint.toBuffer(),
        Buffer.from('edition'),
      ],
      METAPLEX_PROGRAM_ID,
    );
    const accounts: MintInstructionAccounts = {
      collectionAuthorityRecord,
      collectionMasterEdition,
      collectionMetadata,
      collectionMint,
      candyGuard: candyGuard,
      candyMachineProgram: CANDY_MACHINE_PROGRAM,
      candyMachine: candyMachine,
      updateAuthority: candyMachineObject.updateAuthority,
      payer: payer.publicKey,
      candyMachineCreator: candyMachineCreator,
      masterEdition: masterEdition,
      metadata: metadataAddress,
      mint: mint.publicKey,
      mintAuthority: payer.publicKey,
      mintUpdateAuthority: payer.publicKey,
      tokenMetadataProgram: METAPLEX_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      recentSlothashes: SYSVAR_SLOT_HASHES_PUBKEY,
      instructionSysvarAccount: SYSVAR_INSTRUCTIONS_PUBKEY,
    };

    const args: MintInstructionArgs = {
      creatorBump: bump,
      mintArgs: new Uint8Array(),
    };

    const ixs: TransactionInstruction[] = [];
    ixs.push(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: mint.publicKey,
        lamports: await connection.getMinimumBalanceForRentExemption(MintLayout.span),
        space: MintLayout.span,
        programId: TOKEN_PROGRAM_ID,
      }),
    );
    ixs.push(createInitializeMintInstruction(mint.publicKey, 0, payer.publicKey, payer.publicKey));
    ixs.push(
      createAssociatedTokenAccountInstruction(
        payer.publicKey,
        associatedToken,
        payer.publicKey,
        mint.publicKey,
      ),
    );
    ixs.push(createMintToInstruction(mint.publicKey, associatedToken, payer.publicKey, 1, []));
    ixs.push(createMintInstruction(accounts, args));
    const tx = new Transaction().add(...ixs);

    return { tx: handler.sendAndConfirmTransaction(tx, [payer, mint], 'tx: Candy Guard Mint') };
  }

  async deploy(
    t: Test,
    guards: CandyGuardData,
    payer: Keypair,
    handler: PayerTransactionHandler,
    connection: Connection,
    collection?: PublicKey | null,
  ): Promise<{ candyGuard: PublicKey; candyMachine: PublicKey }> {
    // candy machine

    const [, candyMachine] = await amman.genLabeledKeypair('Candy Machine Account');

    const items = 10;

    const candyMachineData = {
      itemsAvailable: items,
      symbol: 'CORE',
      sellerFeeBasisPoints: 500,
      maxSupply: 0,
      isMutable: true,
      retainAuthority: true,
      creators: [
        {
          address: payer.publicKey,
          verified: false,
          percentageShare: 100,
        },
      ],
      configLineSettings: {
        prefixName: 'TEST ',
        nameLength: 10,
        prefixUri: 'https://arweave.net/',
        uriLength: 50,
        isSequential: false,
      },
      hiddenSettings: null,
    };

    const { tx: createTxCM } = await HELPER.create(
      t,
      payer,
      candyMachine,
      candyMachineData,
      handler,
      connection,
    );
    // executes the transaction
    await createTxCM.assertSuccess(t);

    const lines: { name: string; uri: string }[] = [];

    for (let i = 0; i < items; i++) {
      const line = {
        name: `NFT #${i + 1}`,
        uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
      };

      lines.push(line);
    }
    const { txs } = await HELPER.addConfigLines(t, candyMachine.publicKey, payer, lines, handler);
    // confirms that all lines have been written
    for (const tx of txs) {
      await handler.sendAndConfirmTransaction(tx, [payer], 'tx: AddConfigLines').assertNone();
    }

    if (collection) {
      const { tx: addCollectionTx } = await HELPER.setCollection(
        t,
        candyMachine.publicKey,
        collection,
        payer,
        handler,
        connection,
      );
      await addCollectionTx.assertNone();
    }

    // candy guard

    const { tx: initializeTxCG, candyGuard: address } = await this.initialize(
      t,
      guards,
      payer,
      handler,
    );
    // executes the transaction
    await initializeTxCG.assertSuccess(t);

    const { tx: wrapTx } = await this.wrap(t, address, candyMachine.publicKey, payer, handler);

    await wrapTx.assertSuccess(t, [/SetAuthority/i]);

    return { candyGuard: address, candyMachine: candyMachine.publicKey };
  }
}
