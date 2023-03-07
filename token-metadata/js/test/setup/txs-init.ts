import {
  ConfirmedTransactionAssertablePromise,
  GenLabeledKeypair,
  LoadOrGenKeypair,
  LOCALHOST,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman-client';
import * as splToken from '@solana/spl-token';
import {
  ComputeBudgetProgram,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Transaction,
  TransactionInstruction,
} from '@solana/web3.js';
import {
  AssetData,
  createCreateInstruction,
  CreateInstructionAccounts,
  CreateInstructionArgs,
  createMintInstruction,
  MintInstructionAccounts,
  MintInstructionArgs,
  createUpdateInstruction,
  createTransferInstruction,
  UpdateInstructionAccounts,
  UpdateInstructionArgs,
  PROGRAM_ID,
  TokenStandard,
  TransferInstructionAccounts,
  TransferInstructionArgs,
  AuthorizationData,
  Payload,
  SignMetadataInstructionAccounts,
  VerifyCollectionInstructionAccounts,
  createVerifyCollectionInstruction,
  createSignMetadataInstruction,
  Metadata,
  DelegateInstructionAccounts,
  DelegateInstructionArgs,
  DelegateArgs,
  createDelegateInstruction,
  RevokeInstructionAccounts,
  RevokeInstructionArgs,
  createRevokeInstruction,
  RevokeArgs,
  LockInstructionAccounts,
  LockInstructionArgs,
  createLockInstruction,
  UnlockInstructionAccounts,
  UnlockInstructionArgs,
  createUnlockInstruction,
  TransferArgs,
  BurnInstructionAccounts,
  BurnInstructionArgs,
  createBurnInstruction,
  VerifyInstructionAccounts,
  VerifyInstructionArgs,
  createVerifyInstruction,
  UnverifyInstructionAccounts,
  UnverifyInstructionArgs,
  createUnverifyInstruction,
} from '../../src/generated';
import { Test } from 'tape';
import { amman } from '.';
import { UpdateTestData } from '../utils/update-test-data';
import {
  CreateOrUpdateInstructionAccounts,
  CreateOrUpdateInstructionArgs,
  createCreateOrUpdateInstruction,
  PROGRAM_ID as TOKEN_AUTH_RULES_ID,
} from '@metaplex-foundation/mpl-token-auth-rules';
import {
  ACCOUNT_SIZE,
  createInitializeAccountInstruction,
  createInitializeMintInstruction,
  MintLayout,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { findTokenRecordPda } from '../utils/programmable';
import { encode } from '@msgpack/msgpack';

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

  async burn(
    handler: PayerTransactionHandler,
    authority: Keypair,
    mint: PublicKey,
    metadata: PublicKey,
    token: PublicKey,
    amount: number,
    edition: PublicKey | null = null,
    tokenRecord: PublicKey | null = null,
    masterEdition: PublicKey | null = null,
    masterEditionMint: PublicKey | null = null,
    masterEditionToken: PublicKey | null = null,
    editionMarker: PublicKey | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Mint Account', mint);
    amman.addr.addLabel('Metadata Account', metadata);
    if (edition != null) {
      amman.addr.addLabel('Edition Account', edition);
    }

    const burnAccounts: BurnInstructionAccounts = {
      authority: authority.publicKey,
      metadata,
      edition,
      mint,
      token,
      tokenRecord,
      masterEdition,
      masterEditionMint,
      masterEditionToken,
      editionMarker,
      systemProgram: SystemProgram.programId,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
    };

    const burnArgs: BurnInstructionArgs = {
      burnArgs: {
        __kind: 'V1',
        amount,
      },
    };

    const burnIx = createBurnInstruction(burnAccounts, burnArgs);
    const tx = new Transaction().add(burnIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [authority], 'tx: Burn'),
    };
  }

  async verify(
    handler: PayerTransactionHandler,
    authority: Keypair,
    delegateRecord: PublicKey | null = null,
    metadata: PublicKey,
    collectionMint: PublicKey | null = null,
    collectionMetadata: PublicKey | null = null,
    collectionMasterEdition: PublicKey | null = null,
    args: VerifyInstructionArgs,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Metadata Account', metadata);

    const verifyAccounts: VerifyInstructionAccounts = {
      authority: authority.publicKey,
      delegateRecord,
      metadata,
      collectionMint,
      collectionMetadata,
      collectionMasterEdition,
      systemProgram: SystemProgram.programId,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    };

    const verifyIx = createVerifyInstruction(verifyAccounts, args);
    const tx = new Transaction().add(verifyIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [authority], 'tx: Verify'),
    };
  }

  async unverify(
    handler: PayerTransactionHandler,
    authority: Keypair,
    delegateRecord: PublicKey | null = null,
    metadata: PublicKey,
    collectionMint: PublicKey | null = null,
    collectionMetadata: PublicKey | null = null,
    args: UnverifyInstructionArgs,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Metadata Account', metadata);

    const unverifyAccounts: UnverifyInstructionAccounts = {
      authority: authority.publicKey,
      delegateRecord,
      metadata,
      collectionMint,
      collectionMetadata,
      systemProgram: SystemProgram.programId,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    };

    const unverifyIx = createUnverifyInstruction(unverifyAccounts, args);
    const tx = new Transaction().add(unverifyIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [authority], 'tx: Verify'),
    };
  }

  async create(
    t: Test,
    payer: Keypair,
    assetData: AssetData,
    decimals: number,
    printSupply: number,
    handler: PayerTransactionHandler,
    mint: PublicKey | null = null,
    metadata: PublicKey | null = null,
    masterEdition: PublicKey | null = null,
    skipMasterEdition = false,
  ): Promise<{
    tx: ConfirmedTransactionAssertablePromise;
    mint: PublicKey;
    metadata: PublicKey;
    masterEdition?: PublicKey;
  }> {
    let mintPair = null;
    // create a keypair for the mint account (if needed)
    if (!mint) {
      const [, keypair] = await this.getKeypair('Mint Account');
      amman.addr.addLabel('Mint Account', keypair.publicKey);
      mintPair = keypair;
    }

    // metadata account
    if (!metadata) {
      const [address] = PublicKey.findProgramAddressSync(
        [
          Buffer.from('metadata'),
          PROGRAM_ID.toBuffer(),
          mint ? mint.toBuffer() : mintPair.publicKey.toBuffer(),
        ],
        PROGRAM_ID,
      );
      amman.addr.addLabel('Metadata Account', address);
      metadata = address;
    }

    if (
      !masterEdition &&
      (assetData.tokenStandard == TokenStandard.NonFungible ||
        assetData.tokenStandard == TokenStandard.ProgrammableNonFungible) &&
      !skipMasterEdition
    ) {
      // master edition (optional)
      const [address] = PublicKey.findProgramAddressSync(
        [
          Buffer.from('metadata'),
          PROGRAM_ID.toBuffer(),
          mint ? mint.toBuffer() : mintPair.publicKey.toBuffer(),
          Buffer.from('edition'),
        ],
        PROGRAM_ID,
      );
      amman.addr.addLabel('Master Edition Account', address);
      masterEdition = address;
    }

    const accounts: CreateInstructionAccounts = {
      metadata,
      masterEdition,
      mint: mint ? mint : mintPair.publicKey,
      authority: payer.publicKey,
      payer: payer.publicKey,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      updateAuthority: payer.publicKey,
    };

    const args: CreateInstructionArgs = {
      createArgs: {
        __kind: 'V1',
        assetData,
        decimals,
        printSupply:
          printSupply == 0 ? { __kind: 'Zero' } : { __kind: 'Limited', fields: [printSupply] },
      },
    };

    const createIx = createCreateInstruction(accounts, args);

    if (!mint) {
      // this test always initializes the mint, we we need to set the
      // account to be writable and a signer
      for (let i = 0; i < createIx.keys.length; i++) {
        if (createIx.keys[i].pubkey.toBase58() === mintPair.publicKey.toBase58()) {
          createIx.keys[i].isSigner = true;
          createIx.keys[i].isWritable = true;
        }
      }
    }

    const tx = new Transaction().add(createIx);
    const signers = [payer];
    if (!mint) {
      signers.push(mintPair);
    }

    return {
      tx: handler.sendAndConfirmTransaction(tx, signers, 'tx: Create'),
      mint: mint ? mint : mintPair.publicKey,
      metadata,
      masterEdition,
    };
  }

  async mint(
    t: Test,
    connection: Connection,
    payer: Keypair,
    mint: PublicKey,
    metadata: PublicKey,
    masterEdition: PublicKey,
    authorizationData: AuthorizationData,
    amount: number,
    handler: PayerTransactionHandler,
    token: PublicKey | null = null,
    tokenRecord: PublicKey | null = null,
    tokenOwner: PublicKey | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; token: PublicKey }> {
    if (!token) {
      // mint instrution will initialize a ATA account
      const [tokenPda] = PublicKey.findProgramAddressSync(
        [payer.publicKey.toBuffer(), splToken.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      );
      token = tokenPda;
    }

    if (!tokenOwner) {
      tokenOwner = payer.publicKey;
    }

    if (!tokenRecord) {
      tokenRecord = findTokenRecordPda(mint, token);
    }

    amman.addr.addLabel('Token Account', token);

    const metadataAccount = await Metadata.fromAccountAddress(connection, metadata);
    const authConfig = metadataAccount.programmableConfig;

    const mintAcccounts: MintInstructionAccounts = {
      token,
      tokenOwner,
      metadata,
      masterEdition,
      tokenRecord,
      mint,
      payer: payer.publicKey,
      authority: payer.publicKey,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      splAtaProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      authorizationRules: authConfig ? authConfig.ruleSet : null,
      authorizationRulesProgram: TOKEN_AUTH_RULES_ID,
    };

    const payload: Payload = {
      map: new Map(),
    };

    if (!authorizationData) {
      authorizationData = {
        payload,
      };
    }

    const mintArgs: MintInstructionArgs = {
      mintArgs: {
        __kind: 'V1',
        amount,
        authorizationData,
      },
    };

    const mintIx = createMintInstruction(mintAcccounts, mintArgs);

    // creates the transaction

    const tx = new Transaction().add(mintIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Mint'),
      token,
    };
  }

  async transfer(
    authority: Keypair,
    tokenOwner: PublicKey,
    token: PublicKey,
    mint: PublicKey,
    metadata: PublicKey,
    edition: PublicKey,
    destinationOwner: PublicKey,
    destination: PublicKey,
    authorizationRules: PublicKey,
    amount: number,
    handler: PayerTransactionHandler,
    tokenRecord: PublicKey | null = null,
    destinationTokenRecord: PublicKey | null = null,
    args: TransferArgs | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Mint Account', mint);
    amman.addr.addLabel('Metadata Account', metadata);
    if (edition != null) {
      amman.addr.addLabel('Master Edition Account', edition);
    }
    amman.addr.addLabel('Authority', authority.publicKey);
    amman.addr.addLabel('Token Owner', tokenOwner);
    amman.addr.addLabel('Token Account', token);
    amman.addr.addLabel('Destination', destinationOwner);
    amman.addr.addLabel('Destination Token Account', destination);

    const transferAcccounts: TransferInstructionAccounts = {
      authority: authority.publicKey,
      tokenOwner,
      token,
      metadata,
      mint,
      edition,
      destinationOwner,
      destination,
      payer: authority.publicKey,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      splAtaProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      authorizationRules,
      authorizationRulesProgram: TOKEN_AUTH_RULES_ID,
      ownerTokenRecord: tokenRecord,
      destinationTokenRecord,
    };

    if (!args) {
      args = {
        __kind: 'V1',
        amount,
        authorizationData: null,
      };
    }

    const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
      units: 400_000,
    });

    const transferArgs: TransferInstructionArgs = {
      transferArgs: args,
    };

    const transferIx = createTransferInstruction(transferAcccounts, transferArgs);

    const tx = new Transaction().add(modifyComputeUnits).add(transferIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [authority], 'tx: Transfer'),
    };
  }

  async update(
    t: Test,
    handler: PayerTransactionHandler,
    mint: PublicKey,
    metadata: PublicKey,
    authority: Keypair,
    updateTestData: UpdateTestData,
    delegateRecord: PublicKey | null = null,
    masterEdition: PublicKey | null = null,
    token: PublicKey | null = null,
    ruleSetPda?: PublicKey | null,
    authorizationData?: AuthorizationData | null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Mint Account', mint);
    amman.addr.addLabel('Metadata Account', metadata);
    if (masterEdition != null) {
      amman.addr.addLabel('Edition Account', masterEdition);
    }

    const updateAcccounts: UpdateInstructionAccounts = {
      metadata,
      edition: masterEdition,
      mint,
      systemProgram: SystemProgram.programId,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      authority: authority.publicKey,
      payer: authority.publicKey,
      token,
      delegateRecord,
      authorizationRulesProgram: ruleSetPda ? TOKEN_AUTH_RULES_ID : PROGRAM_ID,
      authorizationRules: ruleSetPda,
    };

    const updateArgs: UpdateInstructionArgs = {
      updateArgs: {
        __kind: 'V1',
        newUpdateAuthority: updateTestData.newUpdateAuthority,
        data: updateTestData.data,
        primarySaleHappened: updateTestData.primarySaleHappened,
        isMutable: updateTestData.isMutable,
        collection: updateTestData.collection,
        uses: updateTestData.uses,
        collectionDetails: updateTestData.collectionDetails,
        ruleSet: updateTestData.ruleSet,
        authorizationData,
      },
    };

    const updateIx = createUpdateInstruction(updateAcccounts, updateArgs);

    const tx = new Transaction().add(updateIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [authority], 'tx: Update'),
    };
  }

  async delegate(
    delegate: PublicKey,
    mint: PublicKey,
    metadata: PublicKey,
    authority: PublicKey,
    payer: Keypair,
    args: DelegateArgs,
    handler: PayerTransactionHandler,
    delegateRecord: PublicKey | null = null,
    masterEdition: PublicKey | null = null,
    token: PublicKey | null = null,
    tokenRecord: PublicKey | null = null,
    ruleSetPda: PublicKey | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const delegateAcccounts: DelegateInstructionAccounts = {
      delegateRecord,
      delegate,
      metadata,
      masterEdition,
      tokenRecord,
      mint,
      token,
      authority,
      payer: payer.publicKey,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      authorizationRulesProgram: TOKEN_AUTH_RULES_ID,
      authorizationRules: ruleSetPda,
    };

    const mintArgs: DelegateInstructionArgs = {
      delegateArgs: args,
    };

    const mintIx = createDelegateInstruction(delegateAcccounts, mintArgs);

    // creates the transaction

    const tx = new Transaction().add(mintIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Delegate'),
    };
  }

  async revoke(
    delegate: PublicKey,
    mint: PublicKey,
    metadata: PublicKey,
    authority: Keypair,
    payer: Keypair,
    args: RevokeArgs,
    handler: PayerTransactionHandler,
    delegateRecord: PublicKey | null = null,
    masterEdition: PublicKey | null = null,
    token: PublicKey | null = null,
    tokenRecord: PublicKey | null = null,
    ruleSetPda: PublicKey | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; delegate: PublicKey }> {
    const revokeAcccounts: RevokeInstructionAccounts = {
      delegateRecord,
      delegate,
      metadata,
      masterEdition,
      tokenRecord,
      mint,
      token,
      authority: authority.publicKey,
      payer: payer.publicKey,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      authorizationRulesProgram: TOKEN_AUTH_RULES_ID,
      authorizationRules: ruleSetPda,
    };

    const revokeArgs: RevokeInstructionArgs = {
      revokeArgs: args,
    };

    const mintIx = createRevokeInstruction(revokeAcccounts, revokeArgs);

    // creates the transaction

    const tx = new Transaction().add(mintIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer, authority], 'tx: Revoke'),
      delegate,
    };
  }

  async lock(
    delegate: Keypair,
    mint: PublicKey,
    metadata: PublicKey,
    token: PublicKey,
    payer: Keypair,
    handler: PayerTransactionHandler,
    tokenRecord: PublicKey | null = null,
    tokenOwner: PublicKey | null = null,
    masterEdition: PublicKey | null = null,
    ruleSetPda: PublicKey | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const lockAcccounts: LockInstructionAccounts = {
      authority: delegate.publicKey,
      tokenOwner,
      tokenRecord,
      token,
      mint,
      metadata,
      edition: masterEdition,
      payer: payer.publicKey,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      authorizationRulesProgram: TOKEN_AUTH_RULES_ID,
      authorizationRules: ruleSetPda,
    };

    const lockArgs: LockInstructionArgs = {
      lockArgs: {
        __kind: 'V1',
        authorizationData: null,
      },
    };

    const mintIx = createLockInstruction(lockAcccounts, lockArgs);

    // creates the transaction

    const tx = new Transaction().add(mintIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer, delegate], 'tx: Lock'),
    };
  }

  async unlock(
    delegate: Keypair,
    mint: PublicKey,
    metadata: PublicKey,
    token: PublicKey,
    payer: Keypair,
    handler: PayerTransactionHandler,
    tokenRecord: PublicKey | null = null,
    tokenOwner: PublicKey | null = null,
    masterEdition: PublicKey | null = null,
    ruleSetPda: PublicKey | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    const unlockAcccounts: UnlockInstructionAccounts = {
      authority: delegate.publicKey,
      tokenOwner,
      tokenRecord,
      token,
      mint,
      metadata,
      edition: masterEdition,
      payer: payer.publicKey,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      authorizationRulesProgram: TOKEN_AUTH_RULES_ID,
      authorizationRules: ruleSetPda,
    };

    const unlockArgs: UnlockInstructionArgs = {
      unlockArgs: {
        __kind: 'V1',
        authorizationData: null,
      },
    };

    const mintIx = createUnlockInstruction(unlockAcccounts, unlockArgs);

    // creates the transaction

    const tx = new Transaction().add(mintIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer, delegate], 'tx: Unlock'),
    };
  }

  //--------------------+
  // Helpers            |
  //--------------------+

  async verifyCollection(
    t: Test,
    payer: Keypair,
    metadata: PublicKey,
    collectionMint: PublicKey,
    collectionMetadata: PublicKey,
    collectionMasterEdition: PublicKey,
    collectionAuthority: Keypair,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Metadata Account', metadata);
    amman.addr.addLabel('Collection Mint Account', collectionMint);
    amman.addr.addLabel('Collection Metadata Account', collectionMetadata);
    amman.addr.addLabel('Collection Master Edition Account', collectionMasterEdition);

    const verifyCollectionAcccounts: VerifyCollectionInstructionAccounts = {
      metadata,
      collectionAuthority: collectionAuthority.publicKey,
      collectionMint,
      collection: collectionMetadata,
      collectionMasterEditionAccount: collectionMasterEdition,
      payer: payer.publicKey,
    };

    const verifyInstruction = createVerifyCollectionInstruction(verifyCollectionAcccounts);
    const tx = new Transaction().add(verifyInstruction);

    return {
      tx: handler.sendAndConfirmTransaction(
        tx,
        [payer, collectionAuthority],
        'tx: Verify Collection',
      ),
    };
  }
  async signMetadata(
    t: Test,
    creator: Keypair,
    metadata: PublicKey,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Metadata Account', metadata);

    const signMetadataAcccounts: SignMetadataInstructionAccounts = {
      metadata,
      creator: creator.publicKey,
    };

    const signMetadataInstruction = createSignMetadataInstruction(signMetadataAcccounts);
    const tx = new Transaction().add(signMetadataInstruction);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [creator], 'tx: Sign Metadata'),
    };
  }

  async createRuleSet(
    t: Test,
    payer: Keypair,
    ruleSetPda: PublicKey,
    serializedRuleSet: Uint8Array,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Payer', payer.publicKey);

    const createRuleSetAccounts: CreateOrUpdateInstructionAccounts = {
      ruleSetPda,
      payer: payer.publicKey,
      bufferPda: TOKEN_AUTH_RULES_ID,
    };

    const createRuleSetArgs: CreateOrUpdateInstructionArgs = {
      createOrUpdateArgs: {
        __kind: 'V1',
        serializedRuleSet,
      },
    };

    const createRuleSetInstruction = createCreateOrUpdateInstruction(
      createRuleSetAccounts,
      createRuleSetArgs,
    );
    const tx = new Transaction().add(createRuleSetInstruction);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: CreateOrUpdateRuleSet'),
    };
  }

  async createDefaultRuleSet(
    t: Test,
    handler: PayerTransactionHandler,
    payer: Keypair,
    amount = 1,
  ): Promise<{
    tx: ConfirmedTransactionAssertablePromise;
    ruleSet: PublicKey;
  }> {
    const allowList = [Array.from(PROGRAM_ID.toBytes())];
    const transferRules = {
      All: {
        rules: [
          {
            Amount: {
              amount,
              operator: 2 /* equal */,
              field: 'Amount',
            },
          },
          {
            Any: {
              rules: [
                {
                  ProgramOwnedList: {
                    programs: allowList,
                    field: 'Destination',
                  },
                },
                {
                  ProgramOwnedList: {
                    programs: allowList,
                    field: 'Source',
                  },
                },
                {
                  ProgramOwnedList: {
                    programs: allowList,
                    field: 'Authority',
                  },
                },
              ],
            },
          },
        ],
      },
    };

    const ruleSetName = 'default_ruleset_test';
    const ruleSet = {
      libVersion: 1,
      ruleSetName: ruleSetName,
      owner: Array.from(payer.publicKey.toBytes()),
      operations: {
        'Transfer:TransferDelegate': transferRules,
        'Delegate:Sale': 'Pass',
        'Delegate:Transfer': 'Pass',
        'Delegate:LockedTransfer': 'Pass',
      },
    };
    const serializedRuleSet = encode(ruleSet);

    // find the rule set PDA
    const [ruleSetPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
      TOKEN_AUTH_RULES_ID,
    );

    // creates the rule set
    const { tx: createRuleSetTx } = await this.createRuleSet(
      t,
      payer,
      ruleSetPda,
      serializedRuleSet,
      handler,
    );

    return { tx: createRuleSetTx, ruleSet: ruleSetPda };
  }

  async createMintAccount(
    payer: Keypair,
    connection: Connection,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; mint: PublicKey }> {
    const mint = Keypair.generate();
    amman.addr.addLabel('Mint Account', mint.publicKey);

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

    const tx = new Transaction().add(...ixs);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer, mint], 'tx: Create Mint Account'),
      mint: mint.publicKey,
    };
  }

  async createTokenAccount(
    mint: PublicKey,
    payer: Keypair,
    connection: Connection,
    handler: PayerTransactionHandler,
    owner: PublicKey | null = null,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; token: PublicKey }> {
    const token = Keypair.generate();
    amman.addr.addLabel('Token Account', token.publicKey);

    const tx = new Transaction();
    tx.add(
      // create account
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: token.publicKey,
        space: ACCOUNT_SIZE,
        lamports: await connection.getMinimumBalanceForRentExemption(ACCOUNT_SIZE),
        programId: TOKEN_PROGRAM_ID,
      }),
      // initialize token account
      createInitializeAccountInstruction(token.publicKey, mint, owner),
    );

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer, token], 'tx: Create Token Account'),
      token: token.publicKey,
    };
  }

  async getTransferInstruction(
    authority: Keypair,
    tokenOwner: PublicKey,
    token: PublicKey,
    mint: PublicKey,
    metadata: PublicKey,
    edition: PublicKey,
    destinationOwner: PublicKey,
    destination: PublicKey,
    authorizationRules: PublicKey,
    amount: number,
    handler: PayerTransactionHandler,
    tokenRecord: PublicKey | null = null,
    destinationTokenRecord: PublicKey | null = null,
  ): Promise<{ instruction: TransactionInstruction }> {
    const transferAcccounts: TransferInstructionAccounts = {
      authority: authority.publicKey,
      tokenOwner,
      token,
      metadata,
      mint,
      edition,
      destinationOwner,
      destination,
      payer: authority.publicKey,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      splAtaProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      authorizationRules,
      authorizationRulesProgram: TOKEN_AUTH_RULES_ID,
      ownerTokenRecord: tokenRecord,
      destinationTokenRecord,
    };

    const transferArgs: TransferInstructionArgs = {
      transferArgs: {
        __kind: 'V1',
        amount,
        authorizationData: null,
      },
    };

    const instruction = createTransferInstruction(transferAcccounts, transferArgs);

    return { instruction };
  }
}
