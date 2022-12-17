import {
  ConfirmedTransactionAssertablePromise,
  GenLabeledKeypair,
  LoadOrGenKeypair,
  LOCALHOST,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman-client';
import * as splToken from '@solana/spl-token';
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Transaction,
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
} from '../../src/generated';
import { Test } from 'tape';
import { amman } from '.';
import { UpdateTestData } from 'test/utils/UpdateTestData';

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

  async create(
    t: Test,
    payer: Keypair,
    assetData: AssetData,
    decimals: number,
    maxSupply: number,
    handler: PayerTransactionHandler,
  ): Promise<{
    tx: ConfirmedTransactionAssertablePromise;
    mint: PublicKey;
    metadata: PublicKey;
    masterEdition?: PublicKey;
  }> {
    // mint account
    const [, mint] = await this.getKeypair('Mint Account');
    // metadata account
    const [metadata] = PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
      PROGRAM_ID,
    );
    amman.addr.addLabel('Metadata Account', metadata);

    const accounts: CreateInstructionAccounts = {
      metadata,
      mint: mint.publicKey,
      mintAuthority: payer.publicKey,
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
        maxSupply,
      },
    };

    const createIx = createCreateInstruction(accounts, args);

    let masterEdition = null;

    if (
      assetData.tokenStandard == TokenStandard.NonFungible ||
      assetData.tokenStandard == TokenStandard.ProgrammableNonFungible
    ) {
      // master edition (optional)
      [masterEdition] = PublicKey.findProgramAddressSync(
        [
          Buffer.from('metadata'),
          PROGRAM_ID.toBuffer(),
          mint.publicKey.toBuffer(),
          Buffer.from('edition'),
        ],
        PROGRAM_ID,
      );
      amman.addr.addLabel('Master Edition Account', masterEdition);

      createIx.keys.push({
        pubkey: masterEdition,
        isSigner: false,
        isWritable: true,
      });
    }

    // this test always initializes the mint, we we need to set the
    // account to be writable and a signer
    for (let i = 0; i < createIx.keys.length; i++) {
      if (createIx.keys[i].pubkey.toBase58() === mint.publicKey.toBase58()) {
        createIx.keys[i].isSigner = true;
        createIx.keys[i].isWritable = true;
      }
    }

    const tx = new Transaction().add(createIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer, mint], 'tx: Create'),
      mint: mint.publicKey,
      metadata,
      masterEdition,
    };
  }

  async mint(
    t: Test,
    payer: Keypair,
    mint: PublicKey,
    metadata: PublicKey,
    masterEdition: PublicKey,
    amount: number,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; token: PublicKey }> {
    // token account
    const [token] = PublicKey.findProgramAddressSync(
      [payer.publicKey.toBuffer(), splToken.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    amman.addr.addLabel('Token Account', token);

    const mintAcccounts: MintInstructionAccounts = {
      token,
      metadata,
      mint,
      payer: payer.publicKey,
      authority: payer.publicKey,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      splAtaProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      masterEdition,
    };

    const mintArgs: MintInstructionArgs = {
      mintArgs: {
        __kind: 'V1',
        amount,
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
    owner: Keypair,
    ata: PublicKey,
    mint: PublicKey,
    metadata: PublicKey,
    masterEdition: PublicKey,
    destination: PublicKey,
    destinationAta: PublicKey,
    amount: number,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Mint Account', mint);
    amman.addr.addLabel('Metadata Account', metadata);
    if (masterEdition != null) {
      amman.addr.addLabel('Master Edition Account', masterEdition);
    }
    amman.addr.addLabel('Owner', owner.publicKey);
    amman.addr.addLabel('Token Account', ata);
    amman.addr.addLabel('Destination', destination);
    amman.addr.addLabel('Destination Token Account', destinationAta);

    const transferAcccounts: TransferInstructionAccounts = {
      owner: owner.publicKey,
      ata,
      metadata,
      mint,
      edition: masterEdition,
      destination,
      destinationAta,
      splTokenProgram: splToken.TOKEN_PROGRAM_ID,
      splAtaProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    };

    const transferArgs: TransferInstructionArgs = {
      transferArgs: {
        __kind: 'V1',
        amount,
        authorizationData: null,
      },
    };

    const transferIx = createTransferInstruction(transferAcccounts, transferArgs);

    const tx = new Transaction().add(transferIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [owner], 'tx: Transfer'),
    };
  }

  async update(
    t: Test,
    payer: Keypair,
    mint: PublicKey,
    metadata: PublicKey,
    masterEdition: PublicKey,
    updateTestData: UpdateTestData,
    handler: PayerTransactionHandler,
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
    amman.addr.addLabel('Mint Account', mint);
    amman.addr.addLabel('Metadata Account', metadata);
    amman.addr.addLabel('Master Edition Account', masterEdition);

    const updateAcccounts: UpdateInstructionAccounts = {
      metadata,
      mint,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      masterEdition,
      updateAuthority: payer.publicKey,
    };

    const updateArgs: UpdateInstructionArgs = {
      updateArgs: {
        __kind: 'V1',
        newUpdateAuthority: updateTestData.newUpdateAuthority,
        data: updateTestData.data,
        primarySaleHappened: updateTestData.primarySaleHappened,
        isMutable: updateTestData.isMutable,
        tokenStandard: updateTestData.tokenStandard,
        collection: updateTestData.collection,
        uses: updateTestData.uses,
        collectionDetails: updateTestData.collectionDetails,
        programmableConfig: updateTestData.programmableConfig,
        delegateState: updateTestData.delegateState,
        authorizationData: updateTestData.authorizationData,
      },
    };

    const updateIx = createUpdateInstruction(updateAcccounts, updateArgs);

    const tx = new Transaction().add(updateIx);

    return {
      tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Update'),
    };
  }
}
