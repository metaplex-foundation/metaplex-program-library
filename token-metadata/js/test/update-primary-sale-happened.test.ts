// import test from 'tape';

// import { UpdatePrimarySaleHappenedViaToken } from '../src/mpl-token-metadata';
// import { killStuckProcess, initMetadata, getMetadataData, dump } from './utils';
import { killStuckProcess } from './utils';

killStuckProcess();

// -----------------
// Success Cases
// -----------------

// TODO: currently failing, I tried to use this similarly to the test inside
// ../program/tests/update_primary_sale_happened_via_token.rs, but running into the below:
//
// Transaction simulation failed: Error processing Instruction 0: invalid account data for instruction
//     Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s invoke [1]
//     Program log: Instruction: Update primary sale via token
//     Program log: Error: InvalidAccountData
//     Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s consumed 1669 of 200000 compute units
//     Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s failed: invalid account data for instruction
//
// test.skip('update-primary-sale-happened-via-token: toggle primarySaleHappened', async (t) => {
//   const { connection, transactionHandler, payer, metadata, initialMetadata, mint } =
//     await initMetadata();

//   t.notOk(initialMetadata.primarySaleHappened, 'initially sale has not happened');
//   const tx = new UpdatePrimarySaleHappenedViaToken(
//     {},
//     {
//       metadata,
//       owner: payer.publicKey,
//       tokenAccount: mint.publicKey,
//     },
//   );
//   await transactionHandler.sendAndConfirmTransaction(tx, [payer]);

//   const updatedMetadata = await getMetadataData(connection, metadata);
//   dump(updatedMetadata);
// });
