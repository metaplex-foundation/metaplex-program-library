import spok from 'spok';
import { AssetData, Metadata, TokenStandard } from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { PublicKey, SystemProgram, Transaction, TransactionInstruction } from '@solana/web3.js';
import {
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  MintLayout,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID } from './utils';

killStuckProcess();

test('Create: ProgrammableNonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    name: 'ProgrammableNonFungible',
    symbol: 'PNF',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
    updateAuthority: payer.publicKey,
    creators: [
      {
        address: payer.publicKey,
        share: 100,
        verified: false,
      },
    ],
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    ruleSet: null,
  };

  const { tx: transaction, metadata: address } = await API.create(t, payer, data, 0, 0, handler);
  // executes the transaction
  await transaction.assertSuccess(t);

  const metadata = await Metadata.fromAccountAddress(connection, address);

  spok(t, metadata, {
    data: {
      sellerFeeBasisPoints: 0,
    },
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
  });

  t.equal(metadata.data.name.replace(/\0+/, ''), 'ProgrammableNonFungible');
  t.equal(metadata.data.symbol.replace(/\0+/, ''), 'PNF');
  t.equal(metadata.data.uri.replace(/\0+/, ''), 'uri');
});

test('Create: ProgrammableNonFungible with existing mint account', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  // initialize a mint account

  const { tx: mintTx, mint } = await API.createMintAccount(payer, connection, handler);
  await mintTx.assertSuccess(t);

  // create the metadata

  const data: AssetData = {
    name: 'ProgrammableNonFungible',
    symbol: 'PNF',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
    updateAuthority: payer.publicKey,
    creators: [
      {
        address: payer.publicKey,
        share: 100,
        verified: false,
      },
    ],
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    ruleSet: null,
  };

  const { tx: transaction, metadata: address } = await API.create(
    t,
    payer,
    data,
    0,
    0,
    handler,
    mint,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  const metadata = await Metadata.fromAccountAddress(connection, address);

  spok(t, metadata, {
    data: {
      sellerFeeBasisPoints: 0,
    },
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
  });

  t.equal(metadata.data.name.replace(/\0+/, ''), 'ProgrammableNonFungible');
  t.equal(metadata.data.symbol.replace(/\0+/, ''), 'PNF');
  t.equal(metadata.data.uri.replace(/\0+/, ''), 'uri');
});

test.only('Create: ProgrammableNonFungible with minted mint account', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  // initialize a mint account and mints one token

  const [mint, mintKeypair] = await amman.genLabeledKeypair('Mint Account');
  const tokenAccount = PublicKey.findProgramAddressSync(
    [payer.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
  )[0];

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
    createAssociatedTokenAccountInstruction(payer.publicKey, tokenAccount, payer.publicKey, mint),
  );
  ixs.push(createMintToInstruction(mint, tokenAccount, payer.publicKey, 1, []));
  // candy machine mint instruction
  const tx = new Transaction().add(...ixs);

  await handler
    .sendAndConfirmTransaction(tx, [payer, mintKeypair], 'tx: Mint One Token')
    .assertSuccess(t);

  // create the metadata

  const data: AssetData = {
    name: 'ProgrammableNonFungible',
    symbol: 'PNF',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
    updateAuthority: payer.publicKey,
    creators: [
      {
        address: payer.publicKey,
        share: 100,
        verified: false,
      },
    ],
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    ruleSet: null,
  };

  const { tx: transaction } = await API.create(t, payer, data, 0, 0, handler, mint);
  // executes the transaction
  await transaction.assertError(t, /Mint supply must be zero/);
});
