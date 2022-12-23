// import spok from 'spok';
// import { AuthorityType, Metadata, ProgrammableConfig, TokenStandard } from '../src/generated';
// import test from 'tape';
// import { InitTransactions, killStuckProcess } from './setup';
// import { createDefaultAsset } from './utils/DigitalAssetManager';
// import { UpdateTestData } from './utils/UpdateTestData';
// import { PublicKey } from '@solana/web3.js';
// import { PROGRAM_ID as TOKEN_AUTH_RULES_ID } from '@metaplex-foundation/mpl-token-auth-rules';
// import { createPassRuleSet } from './utils/programmable';
// import { encode } from '@msgpack/msgpack';

// killStuckProcess();

// test('Update: Update a ProgrammableNonFungible With No Config', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();
//   const authority = payer;
//   const authorityType = AuthorityType.Metadata;

//   const daManager = await createDefaultAsset(
//     t,
//     connection,
//     API,
//     handler,
//     payer,
//     TokenStandard.ProgrammableNonFungible,
//   );
//   const { mint, metadata, masterEdition } = daManager;

//   const updateData = new UpdateTestData();
//   updateData.data = {
//     name: 'new-name',
//     symbol: 'new-symbol',
//     uri: 'new-uri',
//     sellerFeeBasisPoints: 500,
//     creators: null,
//   };

//   const { tx: updateTx } = await API.update(
//     t,
//     handler,
//     mint,
//     metadata,
//     masterEdition,
//     authority,
//     authorityType,
//     updateData,
//   );
//   await updateTx.assertSuccess(t);

//   const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);
//   spok(t, updatedMetadata.data, {
//     sellerFeeBasisPoints: updateData.data.sellerFeeBasisPoints,
//     creators: updateData.data.creators,
//   });

//   t.equal(updatedMetadata.data.name.replace(/\0+/, ''), updateData.data.name);
//   t.equal(updatedMetadata.data.symbol.replace(/\0+/, ''), updateData.data.symbol);
//   t.equal(updatedMetadata.data.uri.replace(/\0+/, ''), updateData.data.uri);
// });

// test('Update: Update a ProgrammableNonFungible With Config Set', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

//   const authority = payer;
//   const authorityType = AuthorityType.Metadata;

//   const ruleSetName = 'UpdateTest';

//   // Find the ruleset PDA
//   const [ruleSetPda] = PublicKey.findProgramAddressSync(
//     [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
//     TOKEN_AUTH_RULES_ID,
//   );

//   const passRuleSet = createPassRuleSet(ruleSetName, payer.publicKey, 'Update');

//   // Create the ruleset at the PDA address with the serialized ruleset values.
//   const { tx: createRuleSetTx } = await API.createRuleSet(
//     t,
//     payer,
//     ruleSetPda,
//     passRuleSet,
//     handler,
//   );
//   await createRuleSetTx.assertSuccess(t);

//   // // Set up our programmable config with the ruleset PDA.
//   const programmableConfig: ProgrammableConfig = {
//     ruleSet: ruleSetPda,
//   };

//   const daManager = await createDefaultAsset(
//     t,
//     connection,
//     API,
//     handler,
//     payer,
//     TokenStandard.ProgrammableNonFungible,
//     programmableConfig,
//   );
//   const { mint, metadata, masterEdition } = daManager;

//   const updateData = new UpdateTestData();
//   updateData.data = {
//     name: 'new-name',
//     symbol: 'new-symbol',
//     uri: 'new-uri',
//     sellerFeeBasisPoints: 500,
//     creators: null,
//   };

//   const authorizationData = {
//     payload: {
//       map: new Map(),
//     },
//   };

//   const { tx: updateTx } = await API.update(
//     t,
//     handler,
//     mint,
//     metadata,
//     masterEdition,
//     authority,
//     authorityType,
//     updateData,
//     ruleSetPda,
//     authorizationData,
//   );
//   await updateTx.assertSuccess(t);

//   const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);
//   spok(t, updatedMetadata.data, {
//     sellerFeeBasisPoints: updateData.data.sellerFeeBasisPoints,
//     creators: updateData.data.creators,
//   });

//   t.equal(updatedMetadata.data.name.replace(/\0+/, ''), updateData.data.name);
//   t.equal(updatedMetadata.data.symbol.replace(/\0+/, ''), updateData.data.symbol);
//   t.equal(updatedMetadata.data.uri.replace(/\0+/, ''), updateData.data.uri);
// });

// test('Update: Update a ProgrammableNonFungible Where User Adds Rules', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

//   const authority = payer;
//   const authorityType = AuthorityType.Metadata;

//   const ruleSetName = 'UpdateTest';

//   // Find the ruleset PDA
//   const [ruleSetPda] = PublicKey.findProgramAddressSync(
//     [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
//     TOKEN_AUTH_RULES_ID,
//   );

//   const ruleSet = {
//     ruleSetName,
//     owner: Array.from(payer.publicKey.toBytes()),
//     operations: {
//       Update: {
//         PubkeyMatch: {
//           pubkey: Array.from(payer.publicKey.toBytes()),
//           field: 'Target',
//         },
//       },
//     },
//   };
//   const serializedRuleSet = encode(ruleSet);

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

//   const daManager = await createDefaultAsset(
//     t,
//     connection,
//     API,
//     handler,
//     payer,
//     TokenStandard.ProgrammableNonFungible,
//     programmableConfig,
//   );
//   const { mint, metadata, masterEdition } = daManager;

//   const updateData = new UpdateTestData();
//   updateData.data = {
//     name: 'new-name',
//     symbol: 'new-symbol',
//     uri: 'new-uri',
//     sellerFeeBasisPoints: 500,
//     creators: null,
//   };

//   const authorizationData = {
//     payload: {
//       map: new Map(),
//     },
//   };

//   const { tx: updateTx } = await API.update(
//     t,
//     handler,
//     payer,
//     mint,
//     metadata,
//     masterEdition,
//     updateData,
//     ruleSetPda,
//     authorizationData,
//   );
//   await updateTx.assertSuccess(t);

//   const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);
//   spok(t, updatedMetadata.data, {
//     sellerFeeBasisPoints: updateData.data.sellerFeeBasisPoints,
//     creators: updateData.data.creators,
//   });

//   t.equal(updatedMetadata.data.name.replace(/\0+/, ''), updateData.data.name);
//   t.equal(updatedMetadata.data.symbol.replace(/\0+/, ''), updateData.data.symbol);
//   t.equal(updatedMetadata.data.uri.replace(/\0+/, ''), updateData.data.uri);
// });
