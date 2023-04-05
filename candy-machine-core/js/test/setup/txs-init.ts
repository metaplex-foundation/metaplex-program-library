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
  SYSVAR_INSTRUCTIONS_PUBKEY,
  SYSVAR_SLOT_HASHES_PUBKEY,
  Transaction,
  TransactionInstruction,
  ComputeBudgetProgram,
} from '@solana/web3.js';
import {
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  MintLayout,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { Test } from 'tape';
import * as program from '../../src/generated';
import { AccountVersion, CandyMachine, CandyMachineData } from '../../src/generated';
import { amman } from '.';
import { COLLECTION_METADATA, getCandyMachineSpace } from '../utils';
import { keypairIdentity, Metaplex, NftWithToken } from '@metaplex-foundation/js';
import { TokenStandard } from '@metaplex-foundation/mpl-token-metadata';
import { BN } from 'bn.js';

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

  async initialize(
    t: Test,
    payer: Keypair,
    data: program.CandyMachineData,
    handler: PayerTransactionHandler,
    connection: Connection,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; candyMachine: PublicKey }> {
    // creates a collection nft
    const metaplex = Metaplex.make(connection).use(keypairIdentity(payer));

    const { nft: collection } = await metaplex.nfts().create({
      uri: COLLECTION_METADATA,
      name: 'CORE Collection',
      sellerFeeBasisPoints: 500,
    });

    const [, candyMachine] = await this.getKeypair('Candy Machine Account');
    const authorityPda = metaplex
      .candyMachines()
      .pdas()
      .authority({ candyMachine: candyMachine.publicKey });

    await amman.addr.addLabel('Collection Mint', collection.address);

    const collectionAuthorityRecord = metaplex.nfts().pdas().collectionAuthorityRecord({
      mint: collection.mint.address,
      collectionAuthority: authorityPda,
    });
    await amman.addr.addLabel('Collection Authority Record', collectionAuthorityRecord);

    const collectionMetadata = metaplex.nfts().pdas().metadata({ mint: collection.mint.address });
    await amman.addr.addLabel('Collection Metadata', collectionMetadata);

    const collectionMasterEdition = metaplex
      .nfts()
      .pdas()
      .masterEdition({ mint: collection.mint.address });
    await amman.addr.addLabel('Collection Master Edition', collectionMasterEdition);

    const accounts: program.InitializeInstructionAccounts = {
      authorityPda,
      collectionUpdateAuthority: collection.updateAuthorityAddress,
      candyMachine: candyMachine.publicKey,
      authority: payer.publicKey,
      payer: payer.publicKey,
      collectionMetadata,
      collectionMint: collection.address,
      collectionMasterEdition,
      collectionAuthorityRecord,
      tokenMetadataProgram: METAPLEX_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
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

  async initializeV2(
    t: Test,
    payer: Keypair,
    data: program.CandyMachineData,
    tokenStandard: TokenStandard,
    handler: PayerTransactionHandler,
    connection: Connection,
    ruleSet: PublicKey | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; candyMachine: PublicKey }> {
    // creates a collection nft
    const metaplex = Metaplex.make(connection).use(keypairIdentity(payer));

    const { nft: collection } = await metaplex.nfts().create({
      uri: COLLECTION_METADATA,
      name: 'CORE Collection',
      sellerFeeBasisPoints: 500,
    });

    const [, candyMachine] = await this.getKeypair('Candy Machine Account');

    const authorityPda = metaplex
      .candyMachines()
      .pdas()
      .authority({ candyMachine: candyMachine.publicKey });

    await amman.addr.addLabel('Collection Mint', collection.address);

    const collectionMetadata = metaplex.nfts().pdas().metadata({ mint: collection.mint.address });
    await amman.addr.addLabel('Collection Metadata', collectionMetadata);

    const collectionMasterEdition = metaplex
      .nfts()
      .pdas()
      .masterEdition({ mint: collection.mint.address });
    await amman.addr.addLabel('Collection Master Edition', collectionMasterEdition);

    const collectionDelegateRecord = metaplex.nfts().pdas().metadataDelegateRecord({
      mint: collection.address,
      type: 'CollectionV1',
      updateAuthority: payer.publicKey,
      delegate: authorityPda,
    });
    await amman.addr.addLabel('Metadata Delegate Record', collectionDelegateRecord);

    const accounts: program.InitializeV2InstructionAccounts = {
      authorityPda,
      collectionUpdateAuthority: collection.updateAuthorityAddress,
      candyMachine: candyMachine.publicKey,
      authority: payer.publicKey,
      payer: payer.publicKey,
      ruleSet,
      collectionMetadata,
      collectionMint: collection.address,
      collectionMasterEdition,
      collectionDelegateRecord,
      tokenMetadataProgram: METAPLEX_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    };

    const args: program.InitializeV2InstructionArgs = {
      data: data,
      tokenStandard,
    };

    const ixInitialize = program.createInitializeV2Instruction(accounts, args);
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
      'tx: InitializeV2',
    );

    return { tx: txPromise, candyMachine: candyMachine.publicKey };
  }

  async addConfigLines(
    t: Test,
    candyMachine: PublicKey,
    payer: Keypair,
    lines: program.ConfigLine[],
    index: number,
  ): Promise<{ txs: Transaction[] }> {
    const accounts: program.AddConfigLinesInstructionAccounts = {
      candyMachine: candyMachine,
      authority: payer.publicKey,
    };

    const txs: Transaction[] = [];
    let start = 0;

    while (start < lines.length) {
      // sends the config lines in chunks of 10
      const limit = Math.min(lines.length - start, 10);
      const args: program.AddConfigLinesInstructionArgs = {
        configLines: lines.slice(start, start + limit),
        index,
      };

      const ix = program.createAddConfigLinesInstruction(accounts, args);
      txs.push(new Transaction().add(ix));

      start += limit;
      index += limit;
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
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; mintAddress: PublicKey }> {
    const candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);
    // mint address
    const [nftMint, mintPair] = await this.getKeypair('mint');
    await amman.addr.addLabel('NFT Mint', nftMint);

    // PDAs required for the mint

    const metaplex = Metaplex.make(connection).use(keypairIdentity(payer));

    const nftMetadata = metaplex.nfts().pdas().metadata({ mint: nftMint });
    const nftMasterEdition = metaplex.nfts().pdas().masterEdition({ mint: nftMint });
    const nftTokenAccount = metaplex
      .tokens()
      .pdas()
      .associatedTokenAccount({ mint: nftMint, owner: payer.publicKey });

    const collectionMint = candyMachineObject.collectionMint;
    // retrieves the collection nft
    const collection = await metaplex.nfts().findByMint({ mintAddress: collectionMint });
    // collection PDAs
    const authorityPda = metaplex.candyMachines().pdas().authority({ candyMachine });
    const collectionAuthorityRecord = metaplex.nfts().pdas().collectionAuthorityRecord({
      mint: collectionMint,
      collectionAuthority: authorityPda,
    });

    const collectionMetadata = metaplex.nfts().pdas().metadata({ mint: collectionMint });
    const collectionMasterEdition = metaplex.nfts().pdas().masterEdition({ mint: collectionMint });

    const accounts: program.MintInstructionAccounts = {
      candyMachine: candyMachine,
      authorityPda,
      mintAuthority: candyMachineObject.mintAuthority,
      payer: payer.publicKey,
      nftMint,
      nftMintAuthority: payer.publicKey,
      nftMetadata,
      nftMasterEdition,
      collectionAuthorityRecord,
      collectionMint,
      collectionUpdateAuthority: collection.updateAuthorityAddress,
      collectionMetadata,
      collectionMasterEdition,
      tokenMetadataProgram: METAPLEX_PROGRAM_ID,
      recentSlothashes: SYSVAR_SLOT_HASHES_PUBKEY,
    };

    const ixs: TransactionInstruction[] = [];
    const features = new BN(candyMachineObject.features).toBuffer();

    if (!features[0]) {
      // minting NFT
      ixs.push(
        SystemProgram.createAccount({
          fromPubkey: payer.publicKey,
          newAccountPubkey: nftMint,
          lamports: await connection.getMinimumBalanceForRentExemption(MintLayout.span),
          space: MintLayout.span,
          programId: TOKEN_PROGRAM_ID,
        }),
      );
      ixs.push(createInitializeMintInstruction(nftMint, 0, payer.publicKey, payer.publicKey));
      ixs.push(
        createAssociatedTokenAccountInstruction(
          payer.publicKey,
          nftTokenAccount,
          payer.publicKey,
          nftMint,
        ),
      );
      ixs.push(createMintToInstruction(nftMint, nftTokenAccount, payer.publicKey, 1, []));
    }

    // candy machine mint instruction
    const mintIx = program.createMintInstruction(accounts);

    if (features[0]) {
      // minting pNFT
      const remainingAccounts = [];

      // token account
      remainingAccounts.push({
        pubkey: nftTokenAccount,
        isSigner: false,
        isWritable: true,
      });

      // token record
      const tokenRecord = metaplex
        .nfts()
        .pdas()
        .tokenRecord({ mint: nftMint, token: nftTokenAccount });
      remainingAccounts.push({
        pubkey: tokenRecord,
        isSigner: false,
        isWritable: true,
      });

      // sysvar instructions
      remainingAccounts.push({
        pubkey: SYSVAR_INSTRUCTIONS_PUBKEY,
        isSigner: false,
        isWritable: false,
      });

      // SPL ATA program
      remainingAccounts.push({
        pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      });

      mintIx.keys.push(...remainingAccounts);

      // this test always initializes the mint, we we need to set the
      // account to be writable and a signer to avoid warnings
      for (let i = 0; i < mintIx.keys.length; i++) {
        if (mintIx.keys[i].pubkey.toBase58() === mintPair.publicKey.toBase58()) {
          mintIx.keys[i].isSigner = true;
          mintIx.keys[i].isWritable = true;
        }
      }

      const data = Buffer.from(
        Uint8Array.of(0, ...new BN(400000).toArray('le', 4), ...new BN(0).toArray('le', 4)),
      );

      const additionalComputeIx: TransactionInstruction = new TransactionInstruction({
        keys: [],
        programId: ComputeBudgetProgram.programId,
        data,
      });

      ixs.push(additionalComputeIx);
    }

    ixs.push(mintIx);
    const tx = new Transaction().add(...ixs);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer, mintPair], 'tx: Mint'),
      mintAddress: nftMint,
    };
  }

  async mintV2(
    t: Test,
    candyMachine: PublicKey,
    payer: Keypair,
    handler: PayerTransactionHandler,
    connection: Connection,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; mintAddress: PublicKey }> {
    const candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);
    // mint address
    const [nftMint, mintPair] = await this.getKeypair('mint');
    await amman.addr.addLabel('NFT Mint', nftMint);

    // PDAs required for the mint

    const metaplex = Metaplex.make(connection).use(keypairIdentity(payer));

    const nftMetadata = metaplex.nfts().pdas().metadata({ mint: nftMint });
    const nftMasterEdition = metaplex.nfts().pdas().masterEdition({ mint: nftMint });
    const nftTokenAccount = metaplex
      .tokens()
      .pdas()
      .associatedTokenAccount({ mint: nftMint, owner: payer.publicKey });

    const authorityPda = metaplex.candyMachines().pdas().authority({ candyMachine });

    const collectionMint = candyMachineObject.collectionMint;
    // retrieves the collection nft
    const collection = await metaplex.nfts().findByMint({ mintAddress: collectionMint });
    // collection PDAs
    const collectionMetadata = metaplex.nfts().pdas().metadata({ mint: collectionMint });
    const collectionMasterEdition = metaplex.nfts().pdas().masterEdition({ mint: collectionMint });

    const collectionDelegateRecord = metaplex.nfts().pdas().metadataDelegateRecord({
      mint: collection.address,
      type: 'CollectionV1',
      updateAuthority: payer.publicKey,
      delegate: authorityPda,
    });
    await amman.addr.addLabel('Metadata Delegate Record', collectionDelegateRecord);

    const accounts: program.MintV2InstructionAccounts = {
      candyMachine: candyMachine,
      authorityPda,
      mintAuthority: candyMachineObject.mintAuthority,
      payer: payer.publicKey,
      nftOwner: payer.publicKey,
      nftMint,
      nftMintAuthority: payer.publicKey,
      nftMetadata,
      nftMasterEdition,
      token: nftTokenAccount,
      collectionDelegateRecord,
      collectionMint,
      collectionUpdateAuthority: collection.updateAuthorityAddress,
      collectionMetadata,
      collectionMasterEdition,
      tokenMetadataProgram: METAPLEX_PROGRAM_ID,
      splTokenProgram: TOKEN_PROGRAM_ID,
      splAtaProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      recentSlothashes: SYSVAR_SLOT_HASHES_PUBKEY,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    };

    if (candyMachineObject.version == AccountVersion.V2) {
      accounts.tokenRecord = metaplex
        .nfts()
        .pdas()
        .tokenRecord({ mint: nftMint, token: nftTokenAccount });
    }

    const ixs: TransactionInstruction[] = [];
    // candy machine mint instruction
    const mintIx = program.createMintV2Instruction(accounts);

    // this test always initializes the mint, we we need to set the
    // account to be writable and a signer to avoid warnings
    for (let i = 0; i < mintIx.keys.length; i++) {
      if (mintIx.keys[i].pubkey.toBase58() === mintPair.publicKey.toBase58()) {
        mintIx.keys[i].isSigner = true;
        mintIx.keys[i].isWritable = true;
      }
    }

    const data = Buffer.from(
      Uint8Array.of(0, ...new BN(400000).toArray('le', 4), ...new BN(0).toArray('le', 4)),
    );

    const additionalComputeIx: TransactionInstruction = new TransactionInstruction({
      keys: [],
      programId: ComputeBudgetProgram.programId,
      data,
    });

    ixs.push(additionalComputeIx);
    ixs.push(mintIx);
    const tx = new Transaction().add(...ixs);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer, mintPair], 'tx: MintV2'),
      mintAddress: nftMint,
    };
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

  async setCollection(
    t: Test,
    payer: Keypair,
    candyMachine: PublicKey,
    collection: PublicKey,
    newCollection: NftWithToken,
    handler: PayerTransactionHandler,
    connection: Connection,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const metaplex = Metaplex.make(connection).use(keypairIdentity(payer));

    const authorityPda = metaplex.candyMachines().pdas().authority({ candyMachine });

    await amman.addr.addLabel('New Collection Mint', newCollection.address);

    const newCollectionAuthorityRecord = metaplex.nfts().pdas().collectionAuthorityRecord({
      mint: newCollection.mint.address,
      collectionAuthority: authorityPda,
    });
    await amman.addr.addLabel('New Collection Authority Record', newCollectionAuthorityRecord);

    const newCollectionMetadata = metaplex
      .nfts()
      .pdas()
      .metadata({ mint: newCollection.mint.address });
    await amman.addr.addLabel('New Collection Metadata', newCollectionMetadata);

    const newCollectionMasterEdition = metaplex
      .nfts()
      .pdas()
      .masterEdition({ mint: newCollection.mint.address });
    await amman.addr.addLabel('New Collection Master Edition', newCollectionMasterEdition);

    // current collection details
    const collectionMetadata = metaplex.nfts().pdas().metadata({ mint: collection });
    const collectionAuthorityRecord = metaplex
      .nfts()
      .pdas()
      .collectionAuthorityRecord({ mint: collection, collectionAuthority: authorityPda });

    const accounts: program.SetCollectionInstructionAccounts = {
      authorityPda,
      candyMachine: candyMachine,
      authority: payer.publicKey,
      payer: payer.publicKey,
      collectionAuthorityRecord,
      collectionMetadata,
      collectionMint: collection,
      newCollectionAuthorityRecord,
      newCollectionMasterEdition,
      newCollectionMetadata,
      newCollectionMint: newCollection.address,
      newCollectionUpdateAuthority: newCollection.updateAuthorityAddress,
      tokenMetadataProgram: METAPLEX_PROGRAM_ID,
    };

    const ix = program.createSetCollectionInstruction(accounts);
    const tx = new Transaction().add(ix);

    const txPromise = handler.sendAndConfirmTransaction(tx, [payer], 'tx: SetCollection');

    return { tx: txPromise };
  }

  async setTokenStandard(
    t: Test,
    payer: Keypair,
    candyMachine: PublicKey,
    candyMachineObject: CandyMachine,
    collectionUpdateAuthority: Keypair,
    tokenStandard: TokenStandard,
    handler: PayerTransactionHandler,
    connection: Connection,
    ruleSet: PublicKey | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const metaplex = Metaplex.make(connection).use(keypairIdentity(payer));

    const authorityPda = metaplex.candyMachines().pdas().authority({ candyMachine });
    amman.addr.addLabel('Authority PDA', authorityPda);

    const collectionMetadata = metaplex
      .nfts()
      .pdas()
      .metadata({ mint: candyMachineObject.collectionMint });
    amman.addr.addLabel('Collection Metadata', authorityPda);

    let collectionAuthorityRecord = null;

    if (candyMachineObject.version == AccountVersion.V1) {
      collectionAuthorityRecord = metaplex.nfts().pdas().collectionAuthorityRecord({
        mint: candyMachineObject.collectionMint,
        collectionAuthority: authorityPda,
      });
      amman.addr.addLabel('Collection Authority Record', collectionAuthorityRecord);
    }

    const collectionDelegateRecord = metaplex.nfts().pdas().metadataDelegateRecord({
      mint: candyMachineObject.collectionMint,
      type: 'CollectionV1',
      updateAuthority: payer.publicKey,
      delegate: authorityPda,
    });
    amman.addr.addLabel('Collection Delegate Record', collectionDelegateRecord);

    const accounts: program.SetTokenStandardInstructionAccounts = {
      authorityPda,
      candyMachine: candyMachine,
      authority: payer.publicKey,
      payer: payer.publicKey,
      ruleSet,
      collectionAuthorityRecord,
      collectionDelegateRecord,
      collectionMetadata,
      collectionMint: candyMachineObject.collectionMint,
      collectionUpdateAuthority: collectionUpdateAuthority.publicKey,
      tokenMetadataProgram: METAPLEX_PROGRAM_ID,
      authorizationRules: program.PROGRAM_ID,
      authorizationRulesProgram: program.PROGRAM_ID,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    };

    const ix = program.createSetTokenStandardInstruction(accounts, {
      tokenStandard,
    });
    const tx = new Transaction().add(ix);

    const txPromise = handler.sendAndConfirmTransaction(
      tx,
      [payer, collectionUpdateAuthority],
      'tx: SetTokenStandard',
    );

    return { tx: txPromise };
  }
}
