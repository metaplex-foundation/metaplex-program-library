import {
  ConfirmedTransactionAssertablePromise,
  GenLabeledKeypair,
  LoadOrGenKeypair,
  LOCALHOST,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman-client';
import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import { Test } from 'tape';
import { amman } from '.';
import { getCandyGuardPDA } from '../utils';
import {
  CandyGuardData,
  createInitializeInstruction,
  createUpdateInstruction,
  InitializeInstructionAccounts,
  InitializeInstructionArgs,
  PROGRAM_ID,
  UpdateInstructionAccounts,
  UpdateInstructionArgs,
} from '../../src/generated';

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
}
