import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { Keypair, PublicKey, sendAndConfirmTransaction, Transaction } from '@solana/web3.js';
import { createAndMintDefaultAsset } from './utils/digital-asset-manager';
import {
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  getAccount,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import * as splToken from '@solana/spl-token';
import {
  Metadata,
  DelegateArgs,
  TokenStandard,
  TokenRecord,
  TokenDelegateRole,
} from '../src/generated';
import { PROGRAM_ID as TOKEN_AUTH_RULES_ID } from '@metaplex-foundation/mpl-token-auth-rules';
import { PROGRAM_ID as TOKEN_METADATA_ID } from '../src/generated';
import { encode } from '@msgpack/msgpack';
import spok from 'spok';
import { spokSameBignum, spokSamePubkey } from './utils';
import { findTokenRecordPda } from './utils/programmable';

killStuckProcess();

test('Transfer: NonFungible', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
  );

  const owner = payer;
  const destination = Keypair.generate();
  const destinationToken = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );

  const amountBeforeTransfer = destinationToken.amount;

  // transfer

  const amount = 1;

  const { tx: transferTx } = await API.transfer(
    payer,
    owner.publicKey,
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken.address,
    null,
    amount,
    handler,
  );

  await transferTx.assertSuccess(t);

  // asserts

  const amountAfterTransfer = (await getAccount(connection, destinationToken.address)).amount;
  const remainingAmount = (await getAccount(connection, token)).amount;

  t.true(
    amountAfterTransfer > amountBeforeTransfer,
    'amount after transfer is greater than before',
  );
  t.true(amountAfterTransfer.toString() === '1', 'destination amount equal to 1');
  t.true(remainingAmount.toString() === '0', 'source amount equal to 0');
});

test('Transfer: ProgrammableNonFungible (wallet-to-wallet)', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const owner = payer;
  const authority = payer;
  const destination = Keypair.generate();
  const invalidDestination = Keypair.generate();

  amman.airdrop(connection, destination.publicKey, 1);
  amman.airdrop(connection, invalidDestination.publicKey, 1);

  // Set up our rule set with one pubkey match rule for transfer.

  const ruleSetName = 'transfer_test';
  const ruleSet = {
    libVersion: 1,
    ruleSetName: ruleSetName,
    owner: Array.from(owner.publicKey.toBytes()),
    operations: {
      Transfer: {
        ProgramOwned: {
          program: Array.from(owner.publicKey.toBytes()),
          field: 'Destination',
        },
      },
    },
  };
  const serializedRuleSet = encode(ruleSet);

  // Find the ruleset PDA
  const [ruleSetPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
    TOKEN_AUTH_RULES_ID,
  );

  // Create the ruleset at the PDA address with the serialized ruleset values.
  const { tx: createRuleSetTx } = await API.createRuleSet(
    t,
    payer,
    ruleSetPda,
    serializedRuleSet,
    handler,
  );
  await createRuleSetTx.assertSuccess(t);

  // Create an NFT with the programmable config stored on the metadata.
  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    ruleSetPda,
  );

  const metadataAccount = await Metadata.fromAccountAddress(connection, metadata);
  spok(t, metadataAccount.programmableConfig, {
    ruleSet: spokSamePubkey(ruleSetPda),
  });

  const tokenAccount = await getAccount(connection, token, 'confirmed', TOKEN_PROGRAM_ID);
  t.true(tokenAccount.amount.toString() === '1', 'token account amount equal to 1');

  const destinationToken = await createAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );

  // owner token record
  const ownerTokenRecord = findTokenRecordPda(mint, owner.publicKey);
  amman.addr.addLabel('Owner Token Record', ownerTokenRecord);
  // destination token record
  const destinationTokenRecord = findTokenRecordPda(mint, destination.publicKey);
  amman.addr.addLabel('Destination Token Record', destinationTokenRecord);

  // Transfer the NFT to the destination account, this should work since
  // the destination account is in the ruleset.
  const { tx: transferTx } = await API.transfer(
    authority,
    owner.publicKey,
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken,
    ruleSetPda,
    1,
    handler,
    ownerTokenRecord,
    destinationTokenRecord,
  );

  await transferTx.assertSuccess(t);

  t.true(
    (await getAccount(connection, token)).amount.toString() === '0',
    'token amount after transfer equal to 0',
  );
});

test('Transfer: ProgrammableNonFungible (program-owned)', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const owner = payer;
  const authority = payer;

  // Set up our rule set with one pubkey match rule for transfer
  // where the target is a program-owned account of the Token Metadata
  // program.
  const ruleSetName = 'transfer_test';
  const ruleSet = {
    libVersion: 1,
    ruleSetName: ruleSetName,
    owner: Array.from(owner.publicKey.toBytes()),
    operations: {
      'Transfer:Owner': {
        ProgramOwned: {
          program: Array.from(TOKEN_METADATA_ID.toBytes()),
          field: 'Destination',
        },
      },
    },
  };
  const serializedRuleSet = encode(ruleSet);

  // Find the ruleset PDA
  const [ruleSetPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
    TOKEN_AUTH_RULES_ID,
  );

  // Create the ruleset at the PDA address with the serialized ruleset values.
  const { tx: createRuleSetTx } = await API.createRuleSet(
    t,
    payer,
    ruleSetPda,
    serializedRuleSet,
    handler,
  );
  await createRuleSetTx.assertSuccess(t);

  // Create an NFT with the programmable config stored on the metadata.
  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    ruleSetPda,
  );

  const metadataAccount = await Metadata.fromAccountAddress(connection, metadata);
  spok(t, metadataAccount.programmableConfig, {
    ruleSet: spokSamePubkey(ruleSetPda),
  });

  const tokenAccount = await getAccount(connection, token, 'confirmed', TOKEN_PROGRAM_ID);
  t.true(tokenAccount.amount.toString() === '1', 'token account amount equal to 1');

  // [FAIL] Our first destination is going to be an account owned by the
  // mpl-token-auth-rules program as a convenient program-owned account
  // that is not owned by token-metadata.
  const invalidDestination = ruleSetPda;

  // We have to manually run the create ATA transaction since the helper
  // function from SPL token does not allow creating one for an off-curve
  // address.
  const invalidDestinationToken = await getAssociatedTokenAddress(
    mint,
    invalidDestination,
    true, // Allow off-curve addresses
    splToken.TOKEN_PROGRAM_ID,
    splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  const invalidAtaTx = new Transaction().add(
    createAssociatedTokenAccountInstruction(
      payer.publicKey,
      invalidDestinationToken,
      invalidDestination,
      mint,
      splToken.TOKEN_PROGRAM_ID,
      splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
    ),
  );

  await sendAndConfirmTransaction(connection, invalidAtaTx, [payer]);

  // owner token record
  const ownerTokenRecord = findTokenRecordPda(mint, owner.publicKey);
  amman.addr.addLabel('Owner Token Record', ownerTokenRecord);
  // destination token record
  let destinationTokenRecord = findTokenRecordPda(mint, invalidDestination);
  amman.addr.addLabel('Destination Token Record', destinationTokenRecord);

  // Transfer the NFT to the invalid destination account, this should fail.
  const { tx: invalidTransferTx } = await API.transfer(
    authority,
    owner.publicKey,
    token,
    mint,
    metadata,
    masterEdition,
    invalidDestination,
    invalidDestinationToken,
    ruleSetPda,
    1,
    handler,
    ownerTokenRecord,
    destinationTokenRecord,
  );

  // Cusper matches the error code from mpl-token-auth-rules
  // to a mpl-token-metadata error which gives us the wrong message
  // so we match on the actual log values here instead.
  invalidTransferTx.then((x) =>
    x.assertLogs(t, [/Program Owned check failed/i], {
      txLabel: 'tx: Transfer',
    }),
  );
  await invalidTransferTx.assertError(t);

  // Transfer failed so token should still be present on the original
  // account.
  t.true(
    (await getAccount(connection, token)).amount.toString() === '1',
    'token amount after transfer equal to 1',
  );
  t.true(
    (await getAccount(connection, invalidDestinationToken)).amount.toString() === '0',
    'token amount after transfer equal to 0',
  );

  // [SUCESS] Our valid destination is going to be an account owned by the
  // mpl-token-metadata program. Any one will do so for convenience
  // we just use the existing metadata account.
  const destination = metadata;

  // We have to manually run the create ATA transaction since the helper
  // function from SPL token does not allow creating one for an off-curve
  // address.
  const destinationToken = await getAssociatedTokenAddress(
    mint,
    destination,
    true, // Allow off-curve addresses
    splToken.TOKEN_PROGRAM_ID,
    splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  const ataTx = new Transaction().add(
    createAssociatedTokenAccountInstruction(
      payer.publicKey,
      destinationToken,
      destination,
      mint,
      splToken.TOKEN_PROGRAM_ID,
      splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
    ),
  );

  await sendAndConfirmTransaction(connection, ataTx, [payer]);

  // destination token record
  destinationTokenRecord = findTokenRecordPda(mint, metadata);
  amman.addr.addLabel('Destination Token Record', destinationTokenRecord);

  // Transfer the NFT to the destination account, this should work since
  // the destination account is in the ruleset.
  const { tx: transferTx } = await API.transfer(
    authority,
    owner.publicKey,
    token,
    mint,
    metadata,
    masterEdition,
    destination,
    destinationToken,
    ruleSetPda,
    1,
    handler,
    ownerTokenRecord,
    destinationTokenRecord,
  );

  // Cusper matches the error code from mpl-token-auth-rules
  // to a mpl-token-metadata error which gives us the wrong message
  // so we match on the actual log values here instead.
  await transferTx.assertSuccess(t);

  // Transfer succeed so token should have moved to the destination
  // account.
  t.true(
    (await getAccount(connection, token)).amount.toString() === '0',
    'token amount after transfer equal to 0',
  );
  t.true(
    (await getAccount(connection, destinationToken)).amount.toString() === '1',
    'token amount after transfer equal to 1',
  );
});

/*
test('Transfer: NonFungibleEdition', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

Need to call print instead of mint
  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    API,
    handler,
    payer,
    TokenStandard.NonFungibleEdition,
  );

  const owner = payer;
  const destination = Keypair.generate();
  const destinationToken = await createAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );
  const amount = 1;

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
*/

test('Transfer: Fungible', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.Fungible,
    null,
    100,
  );

  const owner = payer;
  const destination = Keypair.generate();
  const destinationToken = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );

  const amountBeforeTransfer = destinationToken.amount;

  // transfer

  const amount = 5;

  const { tx: transferTx } = await API.transfer(
    payer,
    owner.publicKey,
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken.address,
    null,
    amount,
    handler,
  );

  await transferTx.assertSuccess(t);

  // asserts

  const amountAfterTransfer = (await getAccount(connection, destinationToken.address)).amount;
  const remainingAmount = (await getAccount(connection, token)).amount;

  t.true(
    amountAfterTransfer > amountBeforeTransfer,
    'amount after transfer is greater than before',
  );
  t.true(amountAfterTransfer.toString() === '5', 'destination amount equal to 5');
  t.equal(remainingAmount.toString(), '95', 'remaining amount after transfer is 95');
});

test('Transfer: FungibleAsset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.FungibleAsset,
    null,
    10,
  );

  const owner = payer;
  const destination = Keypair.generate();
  const destinationToken = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );

  const amountBeforeTransfer = destinationToken.amount;

  // transfer

  const amount = 5;

  const { tx: transferTx } = await API.transfer(
    payer,
    owner.publicKey,
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken.address,
    null,
    amount,
    handler,
  );

  await transferTx.assertSuccess(t);

  // asserts

  const amountAfterTransfer = (await getAccount(connection, destinationToken.address)).amount;
  const remainingAmount = (await getAccount(connection, token)).amount;

  t.true(
    amountAfterTransfer > amountBeforeTransfer,
    'amount after transfer is greater than before',
  );
  t.true(amountAfterTransfer.toString() === '5', 'destination amount equal to 5');
  t.equal(remainingAmount.toString(), '5', 'remaining amount after transfer is 5');
});

test('Transfer: NonFungible asset with delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const owner = payer;

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
    null,
    1,
  );

  // Generate the delegate keypair
  const delegate = Keypair.generate();

  const delegateArgs: DelegateArgs = {
    __kind: 'StandardV1',
    amount: 1,
  };

  // Approve delegate
  const { tx: delegateTx } = await API.delegate(
    delegate.publicKey,
    mint,
    metadata,
    payer.publicKey,
    payer,
    delegateArgs,
    handler,
    null,
    masterEdition,
    token,
  );
  await delegateTx.assertSuccess(t);

  const destination = Keypair.generate();
  const destinationToken = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );

  const fakeDelegate = Keypair.generate();

  const amount = 1;

  // [FAIL] Try to transfer with fake delegate. This should fail.
  const { tx: fakeDelegateTransferTx } = await API.transfer(
    fakeDelegate, // Transfer authority: the fake delegate
    payer.publicKey, // Owner of the asset
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken.address,
    null,
    amount,
    handler,
  );

  await fakeDelegateTransferTx.assertError(t, /Invalid authority type/);

  // Transfer using the legitimate delegate
  // Try to transfer with fake delegate. This should fail.
  const { tx: transferTx } = await API.transfer(
    delegate, // Transfer authority: the real delegate
    owner.publicKey, // Owner of the asset
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken.address,
    null,
    amount,
    handler,
  );

  await transferTx.assertSuccess(t);
});

test('Transfer: NonFungible asset with invalid authority', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
    null,
    1,
  );

  // This is not a delegate, owner, or a public key in auth rules.
  // Because this is a NFT not a PNFT, it will fail as an
  // invalid authority, not as a failed auth rules check.
  const invalidAuthority = Keypair.generate();

  const destination = Keypair.generate();
  const destinationToken = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );

  const amount = 1;

  // Try to transfer with fake delegate. This should fail.
  const { tx: fakeDelegateTransferTx } = await API.transfer(
    invalidAuthority, // transfer authority: the invalid authority
    payer.publicKey, // Owner of the asset
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken.address,
    null,
    amount,
    handler,
  );

  await fakeDelegateTransferTx.assertError(t, /Invalid authority type/);
});

test('Transfer: ProgrammableNonFungible asset with invalid authority', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const owner = payer;

  // We add this authority to the rule_set as an "Authority"
  // type, which will allow it to transfer the asset.
  const validAuthority = Keypair.generate();

  // This is not a delegate, owner, or a public key in auth rules.
  const invalidAuthority = Keypair.generate();

  // Set up our rule set
  const ruleSetName = 'transfer_test';
  const ruleSet = {
    libVersion: 1,
    ruleSetName: ruleSetName,
    owner: Array.from(owner.publicKey.toBytes()),
    operations: {
      Transfer: {
        PubkeyMatch: {
          pubkey: Array.from(validAuthority.publicKey.toBytes()),
          field: 'Authority',
        },
      },
    },
  };
  const serializedRuleSet = encode(ruleSet);

  // Find the ruleset PDA
  const [ruleSetPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
    TOKEN_AUTH_RULES_ID,
  );

  // Create the ruleset at the PDA address with the serialized ruleset values.
  const { tx: createRuleSetTx } = await API.createRuleSet(
    t,
    payer,
    ruleSetPda,
    serializedRuleSet,
    handler,
  );
  await createRuleSetTx.assertSuccess(t);

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    ruleSetPda,
    1,
  );

  const destination = Keypair.generate();
  const destinationToken = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );

  const amount = 1;

  // Try to transfer with fake delegate. This should fail.
  const { tx: invalidTransferTx } = await API.transfer(
    invalidAuthority, // transfer authority: the invalid authority
    payer.publicKey, // Owner of the asset
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken.address,
    ruleSetPda,
    amount,
    handler,
  );

  await invalidTransferTx.assertError(t, /Invalid authority type/);
});

test('Transfer: ProgrammableNonFungible (uninitialized wallet-to-wallet)', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const owner = payer;
  const authority = payer;
  const destination = Keypair.generate();
  const invalidDestination = Keypair.generate();

  amman.airdrop(connection, destination.publicKey, 1);
  amman.airdrop(connection, invalidDestination.publicKey, 1);

  // Set up our rule set with one pubkey match rule for transfer.

  const ruleSetName = 'transfer_test';
  const ruleSet = {
    libVersion: 1,
    ruleSetName: ruleSetName,
    owner: Array.from(owner.publicKey.toBytes()),
    operations: {
      Transfer: {
        ProgramOwned: {
          program: Array.from(owner.publicKey.toBytes()),
          field: 'Destination',
        },
      },
    },
  };
  const serializedRuleSet = encode(ruleSet);

  // Find the ruleset PDA
  const [ruleSetPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
    TOKEN_AUTH_RULES_ID,
  );

  // Create the ruleset at the PDA address with the serialized ruleset values.
  const { tx: createRuleSetTx } = await API.createRuleSet(
    t,
    payer,
    ruleSetPda,
    serializedRuleSet,
    handler,
  );
  await createRuleSetTx.assertSuccess(t);

  // Create an NFT with the programmable config stored on the metadata.
  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    ruleSetPda,
  );

  const metadataAccount = await Metadata.fromAccountAddress(connection, metadata);
  spok(t, metadataAccount.programmableConfig, {
    ruleSet: spokSamePubkey(ruleSetPda),
  });

  const tokenAccount = await getAccount(connection, token, 'confirmed', TOKEN_PROGRAM_ID);
  t.true(tokenAccount.amount.toString() === '1', 'token account amount equal to 1');

  const [destinationToken] = PublicKey.findProgramAddressSync(
    [destination.publicKey.toBuffer(), splToken.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  // owner token record
  const ownerTokenRecord = findTokenRecordPda(mint, owner.publicKey);
  amman.addr.addLabel('Owner Token Record', ownerTokenRecord);
  // destination token record
  const destinationTokenRecord = findTokenRecordPda(mint, destination.publicKey);
  amman.addr.addLabel('Destination Token Record', destinationTokenRecord);

  // Transfer the NFT to the destination account, this should work since
  // the destination account is in the ruleset.
  const { tx: transferTx } = await API.transfer(
    authority,
    owner.publicKey,
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken,
    ruleSetPda,
    1,
    handler,
    ownerTokenRecord,
    destinationTokenRecord,
  );

  await transferTx.assertSuccess(t);

  t.true(
    (await getAccount(connection, token)).amount.toString() === '0',
    'token amount after transfer equal to 0',
  );
});

test('Transfer: ProgrammableNonFungible (rule set revision)', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();
  const owner = payer;

  // create a rule set that allows transfers to token metadata (revision 0)

  const ruleSetName = 'transfer_test';
  const ruleSetTokenMetadata = {
    libVersion: 1,
    ruleSetName: ruleSetName,
    owner: Array.from(owner.publicKey.toBytes()),
    operations: {
      'Transfer:TransferDelegate': {
        ProgramOwned: {
          program: Array.from(TOKEN_METADATA_ID.toBytes()),
          field: 'Destination',
        },
      },
      'Transfer:Owner': {
        ProgramOwned: {
          program: Array.from(TOKEN_METADATA_ID.toBytes()),
          field: 'Destination',
        },
      },
    },
  };

  const [ruleSetPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
    TOKEN_AUTH_RULES_ID,
  );

  const { tx: createRuleSetTx } = await API.createRuleSet(
    t,
    payer,
    ruleSetPda,
    encode(ruleSetTokenMetadata),
    handler,
  );
  await createRuleSetTx.assertSuccess(t);

  // creates a pNFT

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    ruleSetPda,
  );

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');
  amman.airdrop(connection, delegate.publicKey, 1);
  // token record PDA
  const tokenRecord = findTokenRecordPda(mint, payer.publicKey);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate.publicKey,
    mint,
    metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    masterEdition,
    token,
    tokenRecord,
    ruleSetPda,
  );

  await delegateTx.assertSuccess(t);

  // checks that the rule set revision has been saved

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate.publicKey),
    delegateRole: TokenDelegateRole.Transfer,
    ruleSetRevision: spokSameBignum(0),
  });

  // updates the rule set to allow transfers only to token auth rules (revision 1)

  const ruleSetTokenAuthRules = {
    libVersion: 1,
    ruleSetName: ruleSetName,
    owner: Array.from(owner.publicKey.toBytes()),
    operations: {
      'Transfer:TransferDelegate': {
        ProgramOwned: {
          program: Array.from(TOKEN_AUTH_RULES_ID.toBytes()),
          field: 'Destination',
        },
      },
      'Transfer:Owner': {
        ProgramOwned: {
          program: Array.from(TOKEN_AUTH_RULES_ID.toBytes()),
          field: 'Destination',
        },
      },
    },
  };

  const { tx: createRuleSetTx2 } = await API.createRuleSet(
    t,
    payer,
    ruleSetPda,
    encode(ruleSetTokenAuthRules),
    handler,
  );
  await createRuleSetTx2.assertSuccess(t);

  // performs a transfer using the delegate to the metadata account, which is
  // allowed by revision 0 (this will work because the revision was saved when
  // we set the delegate)

  const [destinationToken] = PublicKey.findProgramAddressSync(
    [metadata.toBuffer(), splToken.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  // owner token record
  const ownerTokenRecord = findTokenRecordPda(mint, owner.publicKey);
  amman.addr.addLabel('Owner Token Record', ownerTokenRecord);
  // destination token record
  const destinationTokenRecord = findTokenRecordPda(mint, metadata);
  amman.addr.addLabel('Destination Token Record', destinationTokenRecord);

  // Transfer the NFT to the destination account, this should work since
  // the destination account is in the ruleset.
  const { tx: transferTx } = await API.transfer(
    owner,
    owner.publicKey,
    token,
    mint,
    metadata,
    masterEdition,
    metadata,
    destinationToken,
    ruleSetPda,
    1,
    handler,
    ownerTokenRecord,
    destinationTokenRecord,
  );

  await transferTx.assertSuccess(t);

  t.true(
    (await getAccount(connection, token)).amount.toString() === '0',
    'token amount after transfer equal to 0',
  );
});
