import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { Keypair } from '@solana/web3.js';
import { createAndMintDefaultAsset } from './utils/DigitalAssetManager';
import { createAssociatedTokenAccount } from '@solana/spl-token';
import { TokenStandard } from 'src/generated';

killStuckProcess();

test('Transfer: NonFungible', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    API,
    handler,
    payer,
  );

  console.log('created and minted');

  const owner = payer;
  const destination = Keypair.generate();
  const destinationToken = await createAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );
  const amount = 1;

  console.log('created destination token account');

  const { tx: transferTx } = await API.transfer(
    owner,
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken,
    amount,
    handler,
  );

  console.log('transfer tx created');

  await transferTx.assertSuccess(t);
});

// test('Transfer: ProgrammableNonFungible', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

//   const programmableConfig = {
//     ruleset,
//   };

//   const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
//     t,
//     API,
//     handler,
//     payer,
//     TokenStandard.ProgrammableNonFungible,
//     programmableConfig,
//   );

//   const owner = payer;
//   const destination = Keypair.generate();
//   const destinationToken = await createAssociatedTokenAccount(
//     connection,
//     payer,
//     mint,
//     destination.publicKey,
//   );
//   const amount = 1;

//   const { tx: transferTx } = await API.transfer(
//     owner,
//     token,
//     mint,
//     metadata,
//     masterEdition,
//     destination.publicKey,
//     destinationToken,
//     amount,
//     handler,
//   );

//   await transferTx.assertSuccess(t);
// });

// test('Transfer: NonFungibleEdition', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

// Need to call print instead of mint
//   const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
//     t,
//     API,
//     handler,
//     payer,
//     TokenStandard.NonFungibleEdition,
//   );

//   const owner = payer;
//   const destination = Keypair.generate();
//   const destinationToken = await createAssociatedTokenAccount(
//     connection,
//     payer,
//     mint,
//     destination.publicKey,
//   );
//   const amount = 1;

//   const { tx: transferTx } = await API.transfer(
//     owner,
//     token,
//     mint,
//     metadata,
//     masterEdition,
//     destination.publicKey,
//     destinationToken,
//     amount,
//     handler,
//   );

//   await transferTx.assertSuccess(t);
// });

test('Transfer: Fungible', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    API,
    handler,
    payer,
    TokenStandard.Fungible,
    null,
    10,
  );

  const owner = payer;
  const destination = Keypair.generate();
  const destinationToken = await createAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );
  const amount = 5;

  const { tx: transferTx } = await API.transfer(
    owner,
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken,
    amount,
    handler,
  );

  await transferTx.assertSuccess(t);
});

test('Transfer: FungibleAsset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    API,
    handler,
    payer,
    TokenStandard.FungibleAsset,
    null,
    10,
  );

  const owner = payer;
  const destination = Keypair.generate();
  const destinationToken = await createAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );
  const amount = 5;

  const { tx: transferTx } = await API.transfer(
    owner,
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken,
    amount,
    handler,
  );

  await transferTx.assertSuccess(t);
});

// test('Transfer: NonFungible asset with delegate', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

//   const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
//     t,
//     API,
//     handler,
//     payer,
//   );

//   const owner = payer;
//   const destination = Keypair.generate();
//   const destinationToken = await createAssociatedTokenAccount(
//     connection,
//     payer,
//     mint,
//     destination.publicKey,
//   );
//   const amount = 1;

//   // Approve delegate
//   panic('Not implemented');

//   const { tx: transferTx } = await API.transfer(
//     owner,
//     token,
//     mint,
//     metadata,
//     masterEdition,
//     destination.publicKey,
//     destinationToken,
//     amount,
//     handler,
//   );

//   await transferTx.assertSuccess(t);
// });
