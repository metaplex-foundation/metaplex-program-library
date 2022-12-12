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
  PROGRAM_ID,
  TokenStandard,
} from '../../src/generated';
import { Test } from 'tape';
import { amman } from '.';

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
  ): Promise<{ tx: ConfirmedTransactionAssertablePromise; mint: PublicKey; metadata: PublicKey }> {
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

    if (
      assetData.tokenStandard == TokenStandard.NonFungible ||
      assetData.tokenStandard == TokenStandard.ProgrammableNonFungible
    ) {
      // master edition (optional)
      const [masterEdition] = PublicKey.findProgramAddressSync(
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
    };
  }

  async mint(
    t: Test,
    payer: Keypair,
    mint: PublicKey,
    metadata: PublicKey,
    masterEdition: PublicKey,
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
        amount: 1,
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
}
