// import test from 'tape';
// import { amman, InitTransactions, killStuckProcess } from './setup';
// import { Keypair, PublicKey } from '@solana/web3.js';
// import { createAndMintDefaultAsset } from './utils/DigitalAssetManager';
// import { createAssociatedTokenAccount, getAccount, TOKEN_PROGRAM_ID } from '@solana/spl-token';
// import { Metadata, ProgrammableConfig, TokenStandard } from 'src/generated';
// import { PROGRAM_ID as TOKEN_AUTH_RULES_ID } from '@metaplex-foundation/mpl-token-auth-rules';
// import { encode } from '@msgpack/msgpack';

// killStuckProcess();

// test('Transfer: NonFungible', async (t) => {
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

// test('Transfer: ProgrammableNonFungible', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

//   const owner = payer;
//   const authority = payer;
//   const destination = Keypair.generate();
//   const invalidDestination = Keypair.generate();
//   amman.airdrop(connection, destination.publicKey, 1);
//   amman.airdrop(connection, invalidDestination.publicKey, 1);

//   // Set up our rule set with one pubkey match rule for transfer.
//   const ruleSetName = 'transfer_test';
//   const ruleSet = {
//     ruleSetName,
//     owner: Array.from(owner.publicKey.toBytes()),
//     operations: {
//       0: {
//         PubkeyMatch: {
//           pubkey: Array.from(destination.publicKey.toBytes()),
//           field: 'Target',
//         },
//       },
//     },
//   };
//   const serializedRuleSet = encode(ruleSet);

//   // Find the ruleset PDA
//   const [ruleSetPda] = PublicKey.findProgramAddressSync(
//     [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
//     TOKEN_AUTH_RULES_ID,
//   );

//   // Create the ruleset at the PDA address with the serialized ruleset values.
//   const { tx: createRuleSetTx } = await API.createRuleSet(
//     t,
//     payer,
//     ruleSetPda,
//     serializedRuleSet,
//     handler,
//   );
//   await createRuleSetTx.assertSuccess(t);

//   // Set up our programmable config with the ruleset PDA.
//   const programmableConfig: ProgrammableConfig = {
//     ruleSet: ruleSetPda,
//   };

//   // Create an NFT with the programmable config stored on the metadata.
//   const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
//     t,
//     connection,
//     API,
//     handler,
//     payer,
//     TokenStandard.ProgrammableNonFungible,
//     programmableConfig,
//   );

//   const metadataAccount = await Metadata.fromAccountAddress(connection, metadata);
//   t.equals(metadataAccount.programmableConfig, programmableConfig);

//   const tokenAccount = await getAccount(connection, token, 'confirmed', TOKEN_PROGRAM_ID);

//   console.log(tokenAccount.amount);

//   // Create the destination token account.
//   const destinationToken = await createAssociatedTokenAccount(
//     connection,
//     payer,
//     mint,
//     destination.publicKey,
//   );
//   const invalidDestinationToken = await createAssociatedTokenAccount(
//     connection,
//     payer,
//     mint,
//     invalidDestination.publicKey,
//   );
//   const amount = 1;

//   // Transfer the NFT to the destination account, this should work since
//   // the destination account is in the ruleset.
//   const { tx: invalidTransferTx } = await API.transfer(
//     authority,
//     owner.publicKey,
//     token,
//     mint,
//     metadata,
//     masterEdition,
//     invalidDestination.publicKey,
//     invalidDestinationToken,
//     ruleSetPda,
//     amount,
//     handler,
//   );
//   await invalidTransferTx.assertError(t, /Pubkey Match check failed/);
//   // await invalidTransferTx.assertSuccess(t);

//   // Transfer the NFT to the destination account, this should work since
//   // the destination account is in the ruleset.
//   const { tx: transferTx } = await API.transfer(
//     authority,
//     owner.publicKey,
//     token,
//     mint,
//     metadata,
//     masterEdition,
//     destination.publicKey,
//     destinationToken,
//     ruleSetPda,
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

// test('Transfer: Fungible', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

//   const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
//     t,
//     API,
//     handler,
//     payer,
//     TokenStandard.Fungible,
//     null,
//     10,
//   );

//   const owner = payer;
//   const destination = Keypair.generate();
//   const destinationToken = await createAssociatedTokenAccount(
//     connection,
//     payer,
//     mint,
//     destination.publicKey,
//   );
//   const amount = 5;

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

// test('Transfer: FungibleAsset', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

//   const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
//     t,
//     API,
//     handler,
//     payer,
//     TokenStandard.FungibleAsset,
//     null,
//     10,
//   );

//   const owner = payer;
//   const destination = Keypair.generate();
//   const destinationToken = await createAssociatedTokenAccount(
//     connection,
//     payer,
//     mint,
//     destination.publicKey,
//   );
//   const amount = 5;

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
