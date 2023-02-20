import { Metadata, TokenStandard } from '../src/generated';
import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { createAndMintDefaultAsset } from './utils/digital-asset-manager';
import { findTokenRecordPda } from './utils/programmable';
import { getAccount, TOKEN_PROGRAM_ID } from '@solana/spl-token';

killStuckProcess();

test('Burn: NonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
  );

  const { mint, metadata, masterEdition, token } = daManager;

  const authority = payer;
  const amount = 1;

  const { tx: updateTx } = await API.burn(
    handler,
    authority,
    mint,
    metadata,
    token,
    amount,
    masterEdition,
  );

  await updateTx.assertSuccess(t);

  // All three accounts are closed.
  const metadataAccount = await connection.getAccountInfo(metadata);
  const editionAccount = await connection.getAccountInfo(masterEdition);
  const tokenAccount = await connection.getAccountInfo(token);

  t.equal(metadataAccount, null);
  t.equal(editionAccount, null);
  t.equal(tokenAccount, null);
});

test('Burn: ProgrammableNonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );
  const { mint, metadata, masterEdition, token } = daManager;

  const tokenRecord = findTokenRecordPda(mint, token);

  const authority = payer;
  const amount = 1;

  const { tx: updateTx } = await API.burn(
    handler,
    authority,
    mint,
    metadata,
    token,
    amount,
    masterEdition,
    tokenRecord,
  );

  await updateTx.assertSuccess(t);

  // All three accounts are closed.
  const metadataAccount = await connection.getAccountInfo(metadata);
  const editionAccount = await connection.getAccountInfo(masterEdition);
  const tokenAccount = await connection.getAccountInfo(token);

  t.equal(metadataAccount, null);
  t.equal(editionAccount, null);
  t.equal(tokenAccount, null);
});

test('Burn: Fungible', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const tokenAmount = 10;

  const daManager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.Fungible,
    null,
    tokenAmount,
  );

  const { mint, metadata, token } = daManager;

  const authority = payer;
  const burnAmount = 1;

  const { tx: burnTx1 } = await API.burn(handler, authority, mint, metadata, token, burnAmount);
  await burnTx1.assertSuccess(t);

  // Metadata and token accounts are open and correct number of tokens remaining.
  const md = await Metadata.fromAccountAddress(connection, metadata);
  const tokenAccount = await getAccount(connection, token, 'confirmed', TOKEN_PROGRAM_ID);

  const remainingAmount = tokenAmount - burnAmount;

  t.equals(md.mint.toString(), mint.toString());
  t.true(
    tokenAccount.amount.toString() === remainingAmount.toString(),
    'token account amount equal to 9',
  );

  const { tx: burnTx2 } = await API.burn(
    handler,
    authority,
    mint,
    metadata,
    token,
    remainingAmount,
  );
  await burnTx2.assertSuccess(t);

  // Metadata account should still be open but token account should be closed.
  const md2 = await Metadata.fromAccountAddress(connection, metadata);
  const tokenAccount2 = await connection.getAccountInfo(token);

  t.equals(md2.mint.toString(), mint.toString());
  t.equal(tokenAccount2, null);
});

test('Burn: Fungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const tokenAmount = 10;

  const daManager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.FungibleAsset,
    null,
    tokenAmount,
  );

  const { mint, metadata, token } = daManager;

  const authority = payer;
  const burnAmount = 1;

  const { tx: burnTx1 } = await API.burn(handler, authority, mint, metadata, token, burnAmount);
  await burnTx1.assertSuccess(t);

  // Metadata and token accounts are open and correct number of tokens remaining.
  const md = await Metadata.fromAccountAddress(connection, metadata);
  const tokenAccount = await getAccount(connection, token, 'confirmed', TOKEN_PROGRAM_ID);

  const remainingAmount = tokenAmount - burnAmount;

  t.equals(md.mint.toString(), mint.toString());
  t.true(
    tokenAccount.amount.toString() === remainingAmount.toString(),
    'token account amount equal to 9',
  );

  const { tx: burnTx2 } = await API.burn(
    handler,
    authority,
    mint,
    metadata,
    token,
    remainingAmount,
  );
  await burnTx2.assertSuccess(t);

  // Metadata account should still be open but token account should be closed.
  const md2 = await Metadata.fromAccountAddress(connection, metadata);
  const tokenAccount2 = await connection.getAccountInfo(token);

  t.equals(md2.mint.toString(), mint.toString());
  t.equal(tokenAccount2, null);
});
