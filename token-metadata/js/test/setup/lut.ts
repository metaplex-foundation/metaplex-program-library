import {
  ConfirmedTransactionAssertablePromise,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman-client';
import {
  AddressLookupTableAccount,
  AddressLookupTableProgram,
  Connection,
  Keypair,
  PublicKey,
  RpcResponseAndContext,
  SignatureResult,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from '@solana/web3.js';

export async function createLookupTable(
  authority: PublicKey,
  payer: Keypair,
  handler: PayerTransactionHandler,
  connection: Connection,
): Promise<{ tx: ConfirmedTransactionAssertablePromise; lookupTable: PublicKey }> {
  // get current `slot`
  let slot = await connection.getSlot();

  // create an Address Lookup Table
  const [lookupTableIx, address] = AddressLookupTableProgram.createLookupTable({
    authority: authority,
    payer: payer.publicKey,
    recentSlot: slot,
  });

  const tx = new Transaction().add(lookupTableIx);
  // send the transaction
  return {
    tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Create Lookup Table'),
    lookupTable: address,
  };
}

export async function addAddressesToTable(
  lookupTable: PublicKey,
  authority: PublicKey,
  payer: Keypair,
  addresses: PublicKey[],
  connection: Connection,
): Promise<{ response: RpcResponseAndContext<SignatureResult>; signature: string }> {
  const addAddressesInstruction = AddressLookupTableProgram.extendLookupTable({
    payer: payer.publicKey,
    authority,
    lookupTable,
    addresses,
  });

  return await createAndSendV0Tx(payer, [addAddressesInstruction], connection);
}

export async function createAndSendV0Tx(
  payer: Keypair,
  instructions: TransactionInstruction[],
  connection: Connection,
  lookupTables: AddressLookupTableAccount[] = [],
): Promise<{ response: RpcResponseAndContext<SignatureResult>; signature: string }> {
  let latestBlockhash = await connection.getLatestBlockhash('finalized');

  const messageV0 = new TransactionMessage({
    payerKey: payer.publicKey,
    recentBlockhash: latestBlockhash.blockhash,
    instructions,
  }).compileToV0Message(lookupTables);

  // creates the versioned transaction
  const transaction = new VersionedTransaction(messageV0);
  //console.log('Transaction size with address lookup: ' + transaction.serialize().length + ' bytes');
  transaction.sign([payer]);

  const signature = await connection.sendTransaction(transaction, { maxRetries: 5 });

  const response = await connection.confirmTransaction({
    signature,
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
  });

  return { response, signature };
}
