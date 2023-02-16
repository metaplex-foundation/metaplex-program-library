import {
  DelegateArgs,
  PROGRAM_ID,
  TokenStandard,
  TokenRecord,
  TokenDelegateRole,
  MetadataDelegateRecord,
  TokenState,
} from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { createAndMintDefaultAsset } from './utils/digital-asset-manager';
import spok from 'spok';
import { spokSameBigint, spokSamePubkey } from './utils';
import { BN } from 'bn.js';
import { getAccount } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { findTokenRecordPda } from './utils/programmable';
import { PROGRAM_ID as TOKEN_AUTH_RULES_ID } from '@metaplex-foundation/mpl-token-auth-rules';
import { encode } from '@msgpack/msgpack';

killStuckProcess();

test('Delegate: create update collection items delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const collection = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [delegate] = await API.getKeypair('Delegate');
  // delegate PDA
  const [delegateRecord] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('metadata'),
      PROGRAM_ID.toBuffer(),
      collection.mint.toBuffer(),
      Buffer.from('collection_delegate'),
      payer.publicKey.toBuffer(),
      delegate.toBuffer(),
    ],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Metadata Delegate Record', delegateRecord);

  const args: DelegateArgs = {
    __kind: 'CollectionV1',
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    collection.mint,
    collection.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    delegateRecord,
    collection.masterEdition,
  );

  await delegateTx.assertSuccess(t);

  const pda = await MetadataDelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    mint: spokSamePubkey(collection.mint),
  });
});

test('Delegate: create sale delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [delegate] = await API.getKeypair('Delegate');
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'SaleV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    delegateRole: TokenDelegateRole.Sale,
    state: TokenState.Listed,
  });
});

test('Delegate: owner as sale delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'SaleV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    payer.publicKey,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(payer.publicKey),
  });

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(payer.publicKey),
    delegateRole: TokenDelegateRole.Sale,
    state: TokenState.Listed,
  });
});

test('Delegate: create transfer delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [delegate] = await API.getKeypair('Delegate');
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    delegateRole: TokenDelegateRole.Transfer,
  });
});

test('Delegate: fail to create sale delegate on NFT', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
  );

  // creates a delegate

  const [delegate] = await API.getKeypair('Delegate');
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'SaleV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertError(t, /Invalid delegate role/);
});

test('Delegate: fail to replace pNFT transfer delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [delegate] = await API.getKeypair('Delegate');
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    delegateRole: TokenDelegateRole.Transfer,
  });

  // creates a new delegate

  const [newDelegate] = await API.getKeypair('Delegate');

  const { tx: delegateTx2 } = await API.delegate(
    newDelegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx2.assertError(t, /Delegate already exists/);
});

test('Delegate: create utility delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [delegate] = await API.getKeypair('Delegate');
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'UtilityV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    delegateRole: TokenDelegateRole.Utility,
  });
});

test('Delegate: try replace sale delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [delegate] = await API.getKeypair('Delegate');
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'SaleV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    delegateRole: TokenDelegateRole.Sale,
  });

  // creates a transfer delegate

  const [newDelegate] = await API.getKeypair('Delegate');

  const args2: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx2 } = await API.delegate(
    newDelegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args2,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx2.assertError(t, /Delegate already exists/);
});

test('Delegate: create locked transfer delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [delegate] = await API.getKeypair('Delegate');
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'LockedTransferV1',
    amount: 1,
    lockedAddress: PublicKey.default,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    delegateRole: TokenDelegateRole.LockedTransfer,
    lockedTransfer: spokSamePubkey(PublicKey.default),
  });
});

test('Delegate: create sale delegate with auth rules', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  // creates the auth rules for the delegate

  const ruleSetName = 'delegate_test';
  const ruleSetTokenMetadata = {
    libVersion: 1,
    ruleSetName: ruleSetName,
    owner: Array.from(payer.publicKey.toBytes()),
    operations: {
      'Delegate:Sale': {
        All: {
          rules: [
            {
              ProgramOwned: {
                program: Array.from(PROGRAM_ID.toBytes()),
                field: 'Delegate',
              },
            },
            {
              Amount: {
                amount: 1,
                operator: 2,
                field: 'Amount',
              },
            },
          ],
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

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    ruleSetPda,
  );

  // creates a delegate

  // token record PDA (using metadata as delegate)
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'SaleV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    manager.metadata,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
    ruleSetPda,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(manager.metadata),
  });

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(manager.metadata),
    delegateRole: TokenDelegateRole.Sale,
    state: TokenState.Listed,
  });
});

test('Delegate: fail to create LockedTransfer delegate with auth rules', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  // creates the auth rules for the delegate

  const ruleSetName = 'delegate_test';
  const ruleSetTokenMetadata = {
    libVersion: 1,
    ruleSetName: ruleSetName,
    owner: Array.from(payer.publicKey.toBytes()),
    operations: {
      'Delegate:LockedTransfer': {
        All: {
          rules: [
            {
              ProgramOwned: {
                program: Array.from(PROGRAM_ID.toBytes()),
                field: 'Delegate',
              },
            },
            {
              Amount: {
                amount: 1,
                operator: 2,
                field: 'Amount',
              },
            },
          ],
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

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    ruleSetPda,
  );

  // creates a delegate

  // address of our invalid delegate
  const [delegate] = await amman.genLabeledKeypair('Delegate');
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const args: DelegateArgs = {
    __kind: 'LockedTransferV1',
    amount: 1,
    lockedAddress: PublicKey.default,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
    ruleSetPda,
  );

  delegateTx.then((x) =>
    x.assertLogs(t, [/Program Owned check failed/i], {
      txLabel: 'tx: Delegate',
    }),
  );
  await delegateTx.assertError(t);
});
