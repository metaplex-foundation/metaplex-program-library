import spok from 'spok';
import { AssetData, Metadata, TokenStandard } from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { PublicKey, SystemProgram, Transaction, TransactionInstruction } from '@solana/web3.js';
import {
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  getMint,
  MintLayout,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, spokSameBigint, spokSamePubkey } from './utils';
import { BN } from 'bn.js';

killStuckProcess();

test('Create: ProgrammableNonFungible', async (t) => {
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

test('Create: fail to create ProgrammableNonFungible with minted mint account', async (t) => {
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

test('Create: failt to create ProgrammableNonFungible with existing metadata account', async (t) => {
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

  const {
    tx: transaction,
    metadata: address,
    mint,
  } = await API.create(t, payer, data, 0, 0, handler);
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

  // tries to create another metadata account to the mint

  const { tx: duplicatedTx } = await API.create(t, payer, data, 0, 0, handler, mint);
  // executes the transaction
  await duplicatedTx.assertError(t, /Mint authority provided does not match the authority/);
});

test('Create: failt to create ProgrammableNonFungible with existing master edition account', async (t) => {
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

  const {
    tx: transaction,
    metadata: address,
    masterEdition,
  } = await API.create(t, payer, data, 0, 0, handler);
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

  // tries to create another metadata account to the mint

  const { tx: duplicatedTx } = await API.create(
    t,
    payer,
    data,
    0,
    0,
    handler,
    null,
    null,
    masterEdition,
  );
  // executes the transaction
  await duplicatedTx.assertError(t, /Derived key invalid/);
});

test('Create: fail to create ProgrammableNonFungible without master edition', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer } = await API.payer();

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

  // tries to create a metadata account

  const { tx: duplicatedTx } = await API.create(
    t,
    payer,
    data,
    0,
    0,
    handler,
    null /* mint */,
    null /* metadata */,
    null /* masterEdition */,
    true /* skip master edition */,
  );
  // executes the transaction
  await duplicatedTx.assertError(t, /Missing master edition account/);
});

test('Create: fail to create NonFungible without master edition', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer } = await API.payer();

  const data: AssetData = {
    name: 'NonFungible',
    symbol: 'NF',
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
    tokenStandard: TokenStandard.NonFungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    ruleSet: null,
  };

  const { tx: transaction } = await API.create(
    t,
    payer,
    data,
    0,
    0,
    handler,
    null /* mint */,
    null /* metadata */,
    null /* masterEdition */,
    true /* skip master edition */,
  );
  // executes the transaction
  await transaction.assertError(t, /Missing master edition account/);
});

test('Create: create NonFungible with minted mint account', async (t) => {
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
    name: 'NonFungible',
    symbol: 'NF',
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
    tokenStandard: TokenStandard.NonFungible,
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
    tokenStandard: TokenStandard.NonFungible,
  });

  t.equal(metadata.data.name.replace(/\0+/, ''), 'NonFungible');
  t.equal(metadata.data.symbol.replace(/\0+/, ''), 'NF');
  t.equal(metadata.data.uri.replace(/\0+/, ''), 'uri');
});

test('Create: fail to create NonFungible with more than 2 mints', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  // initialize a mint account and mints two tokens

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
  ixs.push(createMintToInstruction(mint, tokenAccount, payer.publicKey, 2, []));
  // candy machine mint instruction
  const tx = new Transaction().add(...ixs);

  await handler
    .sendAndConfirmTransaction(tx, [payer, mintKeypair], 'tx: Mint Two Tokens')
    .assertSuccess(t);

  // create the metadata

  const data: AssetData = {
    name: 'NonFungible',
    symbol: 'NF',
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
    tokenStandard: TokenStandard.NonFungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    ruleSet: null,
  };

  const { tx: transaction } = await API.create(t, payer, data, 0, 0, handler, mint);
  // executes the transaction
  await transaction.assertError(t, /Invalid mint account/);
});

test('Create: Fungible', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    name: 'Fungible',
    symbol: 'FUN',
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
    tokenStandard: TokenStandard.Fungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    ruleSet: null,
  };

  const {
    tx: transaction,
    metadata: address,
    mint,
  } = await API.create(t, payer, data, 9, 0, handler);
  // executes the transaction
  await transaction.assertSuccess(t);

  const metadata = await Metadata.fromAccountAddress(connection, address);

  spok(t, metadata, {
    data: {
      sellerFeeBasisPoints: 0,
    },
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.Fungible,
  });

  t.equal(metadata.data.name.replace(/\0+/, ''), 'Fungible');
  t.equal(metadata.data.symbol.replace(/\0+/, ''), 'FUN');
  t.equal(metadata.data.uri.replace(/\0+/, ''), 'uri');

  const mintAccount = await getMint(connection, mint);

  spok(t, mintAccount, {
    decimals: 9,
    supply: spokSameBigint(new BN(0)),
    mintAuthority: spokSamePubkey(payer.publicKey),
  });
});

test('Create: FungibleAsset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    name: 'FungibleAsset',
    symbol: 'FA',
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
    tokenStandard: TokenStandard.FungibleAsset,
    collection: null,
    uses: null,
    collectionDetails: null,
    ruleSet: null,
  };

  const {
    tx: transaction,
    metadata: address,
    mint,
  } = await API.create(t, payer, data, 2, 0, handler);
  // executes the transaction
  await transaction.assertSuccess(t);

  const metadata = await Metadata.fromAccountAddress(connection, address);

  spok(t, metadata, {
    data: {
      sellerFeeBasisPoints: 0,
    },
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.FungibleAsset,
  });

  t.equal(metadata.data.name.replace(/\0+/, ''), 'FungibleAsset');
  t.equal(metadata.data.symbol.replace(/\0+/, ''), 'FA');
  t.equal(metadata.data.uri.replace(/\0+/, ''), 'uri');

  const mintAccount = await getMint(connection, mint);

  spok(t, mintAccount, {
    decimals: 2,
    supply: spokSameBigint(new BN(0)),
    mintAuthority: spokSamePubkey(payer.publicKey),
  });
});

test('Create: create Fungible with minted mint account', async (t) => {
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
  ixs.push(createInitializeMintInstruction(mint, 5, payer.publicKey, payer.publicKey));
  ixs.push(
    createAssociatedTokenAccountInstruction(payer.publicKey, tokenAccount, payer.publicKey, mint),
  );
  ixs.push(createMintToInstruction(mint, tokenAccount, payer.publicKey, 100, []));
  // candy machine mint instruction
  const tx = new Transaction().add(...ixs);

  await handler
    .sendAndConfirmTransaction(tx, [payer, mintKeypair], 'tx: Mint 100 Tokens')
    .assertSuccess(t);

  // create the metadata

  const data: AssetData = {
    name: 'Fungible',
    symbol: 'FUN',
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
    tokenStandard: TokenStandard.Fungible,
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
    tokenStandard: TokenStandard.Fungible,
  });

  t.equal(metadata.data.name.replace(/\0+/, ''), 'Fungible');
  t.equal(metadata.data.symbol.replace(/\0+/, ''), 'FUN');
  t.equal(metadata.data.uri.replace(/\0+/, ''), 'uri');
});

test('Create: create FungibleAsset with minted mint account', async (t) => {
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
  ixs.push(createInitializeMintInstruction(mint, 5, payer.publicKey, payer.publicKey));
  ixs.push(
    createAssociatedTokenAccountInstruction(payer.publicKey, tokenAccount, payer.publicKey, mint),
  );
  ixs.push(createMintToInstruction(mint, tokenAccount, payer.publicKey, 100, []));
  // candy machine mint instruction
  const tx = new Transaction().add(...ixs);

  await handler
    .sendAndConfirmTransaction(tx, [payer, mintKeypair], 'tx: Mint 100 Tokens')
    .assertSuccess(t);

  // create the metadata

  const data: AssetData = {
    name: 'FungibleAsset',
    symbol: 'FA',
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
    tokenStandard: TokenStandard.FungibleAsset,
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
    tokenStandard: TokenStandard.FungibleAsset,
  });

  t.equal(metadata.data.name.replace(/\0+/, ''), 'FungibleAsset');
  t.equal(metadata.data.symbol.replace(/\0+/, ''), 'FA');
  t.equal(metadata.data.uri.replace(/\0+/, ''), 'uri');
});
