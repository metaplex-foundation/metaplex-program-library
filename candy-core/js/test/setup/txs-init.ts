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
  Transaction,
  TransactionInstruction,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_SLOT_HASHES_PUBKEY,
} from '@solana/web3.js';
import {
  MintLayout,
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { Test } from 'tape';
import * as program from '../../src/generated';
import { amman } from '.';
import { getCandyMachineSpace } from '../utils';
import { CandyMachine, CandyMachineData } from '../../src/generated';

const METAPLEX_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

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

  async create(
    t: Test,
    payer: Keypair,
    data: program.CandyMachineData,
    handler: PayerTransactionHandler,
    connection: Connection,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; candyMachine: PublicKey }> {
    const [_, candyMachine] = await this.getKeypair('Candy Machine Account');

    const collectionMint = PublicKey.default;
    amman.addr.addLabel('Collection Mint', collectionMint);

    const updateAuthority = payer.publicKey;

    const [collectionAuthorityRecord] = await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        METAPLEX_PROGRAM_ID.toBuffer(),
        collectionMint.toBuffer(),
        Buffer.from('collection_authority'),
        updateAuthority.toBuffer(),
      ],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Collection Authority Record Master Edition', collectionAuthorityRecord);

    const [collectionMetadata] = await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), METAPLEX_PROGRAM_ID.toBuffer(), collectionMint.toBuffer()],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Collection Metadata', collectionMetadata);

    const [collectionMasterEdition] = await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        METAPLEX_PROGRAM_ID.toBuffer(),
        collectionMint.toBuffer(),
        Buffer.from('edition'),
      ],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Collection Master Edition', collectionMasterEdition);

    const accounts: program.InitializeInstructionAccounts = {
      collectionAuthorityRecord,
      collectionMasterEdition,
      collectionMetadata,
      collectionMint,
      tokenMetadataProgram: candyMachine.publicKey,
      candyMachine: candyMachine.publicKey,
      authority: payer.publicKey,
      updateAuthority: payer.publicKey,
      payer: payer.publicKey,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    };

    const args: program.InitializeInstructionArgs = {
      data: data,
    };

    const ixInitialize = program.createInitializeInstruction(accounts, args);
    const ixCreateAccount = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: candyMachine.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(getCandyMachineSpace(data)),
      space: getCandyMachineSpace(data),
      programId: program.PROGRAM_ID,
    });

    const tx = new Transaction().add(ixCreateAccount).add(ixInitialize);

    const txPromise = handler.sendAndConfirmTransaction(
      tx,
      [candyMachine, payer],
      'tx: Initialize',
    );

    return { tx: txPromise, candyMachine: candyMachine.publicKey };
  }

  async addConfigLines(
    t: Test,
    candyMachine: PublicKey,
    payer: Keypair,
    lines: program.ConfigLine[],
    handler: PayerTransactionHandler,
  ): Promise<{ txs: Transaction[] }> {
    const accounts: program.AddConfigLinesInstructionAccounts = {
      candyMachine: candyMachine,
      authority: payer.publicKey,
    };

    const txs = [];
    let start = 0;

    while (start < lines.length) {
      // sends the config lines in chunks of 10
      const limit = Math.min(lines.length - start, 10);
      const args: program.AddConfigLinesInstructionArgs = {
        configLines: lines.slice(start, start + limit),
        index: start,
      };

      const ix = program.createAddConfigLinesInstruction(accounts, args);
      txs.push(new Transaction().add(ix));

      start = start + limit;
    }

    return { txs };
  }

  async updateCandyMachine(
    t: Test,
    candyMachine: PublicKey,
    payer: Keypair,
    data: CandyMachineData,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const accounts: program.UpdateInstructionAccounts = {
      candyMachine: candyMachine,
      authority: payer.publicKey,
    };

    const args: program.UpdateInstructionArgs = {
      data: data,
    };

    const ix = program.createUpdateInstruction(accounts, args);
    const tx = new Transaction().add(ix);

    return { tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Update') };
  }

  async mint(
    t: Test,
    candyMachine: PublicKey,
    payer: Keypair,
    handler: PayerTransactionHandler,
    connection: Connection,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);
    // mint address
    const [mint, mintPair] = await this.getKeypair('mint');
    amman.addr.addLabel('Mint', mint);

    // PDAs required for the mint

    // creator address
    const [candyMachineCreator, bump] = await PublicKey.findProgramAddress(
      [Buffer.from('candy_machine'), candyMachine.toBuffer()],
      program.PROGRAM_ID,
    );
    amman.addr.addLabel('Mint Creator', candyMachineCreator);

    // associated token address
    const [associatedToken] = await PublicKey.findProgramAddress(
      [payer.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    amman.addr.addLabel('Mint Associated Token', associatedToken);

    // metadata address
    const [metadataAddress] = await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), METAPLEX_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Mint Metadata', metadataAddress);

    // master edition address
    const [masterEdition] = await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        METAPLEX_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
        Buffer.from('edition'),
      ],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Mint Master Edition', masterEdition);

    const collectionMint = PublicKey.default;
    amman.addr.addLabel('Collection Mint', collectionMint);

    const updateAuthority = payer.publicKey;

    const [collectionAuthorityRecord] = await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        METAPLEX_PROGRAM_ID.toBuffer(),
        collectionMint.toBuffer(),
        Buffer.from('collection_authority'),
        updateAuthority.toBuffer(),
      ],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Collection Authority Record Master Edition', collectionAuthorityRecord);

    const [collectionMetadata] = await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), METAPLEX_PROGRAM_ID.toBuffer(), collectionMint.toBuffer()],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Collection Metadata', collectionMetadata);

    const [collectionMasterEdition] = await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        METAPLEX_PROGRAM_ID.toBuffer(),
        collectionMint.toBuffer(),
        Buffer.from('edition'),
      ],
      METAPLEX_PROGRAM_ID,
    );
    amman.addr.addLabel('Collection Master Edition', collectionMasterEdition);

    const accounts: program.MintInstructionAccounts = {
      // TODO: fix the collection accounts
      collectionAuthorityRecord,
      collectionMasterEdition,
      collectionMetadata,
      collectionMint,
      candyMachine: candyMachine,
      authority: candyMachineObject.authority,
      updateAuthority: candyMachineObject.updateAuthority,
      candyMachineCreator: candyMachineCreator,
      masterEdition: masterEdition,
      metadata: metadataAddress,
      mint: mint,
      mintAuthority: payer.publicKey,
      payer: payer.publicKey,
      tokenMetadataProgram: METAPLEX_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      recentSlothashes: SYSVAR_SLOT_HASHES_PUBKEY,
    };

    const args: program.MintInstructionArgs = {
      creatorBump: bump,
    };

    const ixs: TransactionInstruction[] = [];
    ixs.push(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: mint,
        lamports: await connection.getMinimumBalanceForRentExemption(MintLayout.span),
        space: MintLayout.span,
        programId: TOKEN_PROGRAM_ID,
      }),
    );
    ixs.push(createInitializeMintInstruction(mint, 0, payer.publicKey, payer.publicKey));
    ixs.push(
      createAssociatedTokenAccountInstruction(
        payer.publicKey,
        associatedToken,
        payer.publicKey,
        mint,
      ),
    );
    ixs.push(createMintToInstruction(mint, associatedToken, payer.publicKey, 1, []));
    // candy machine mint instruction
    ixs.push(program.createMintInstruction(accounts, args));
    const tx = new Transaction().add(...ixs);

    return { tx: handler.sendAndConfirmTransaction(tx, [payer, mintPair], 'tx: Mint') };
  }

  async withdraw(
    t: Test,
    candyMachine: PublicKey,
    payer: Keypair,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const accounts: program.WithdrawInstructionAccounts = {
      candyMachine: candyMachine,
      authority: payer.publicKey,
    };

    const ix = program.createWithdrawInstruction(accounts);
    const tx = new Transaction().add(ix);

    return { tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Withdraw') };
  }
}
