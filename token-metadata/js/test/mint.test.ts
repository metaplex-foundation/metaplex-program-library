import { getAccount } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { BN } from 'bn.js';
import spok from 'spok';
import { AssetData, PROGRAM_ID, TokenStandard } from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { spokSameBigint } from './utils';
import { DigitalAssetManager } from './utils/digital-asset-manager';

killStuckProcess();

test('Mint: ProgrammableNonFungible', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    name: 'ProgrammableNonFungible',
    symbol: 'PNF',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
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

  const { tx: createTx, metadata, mint } = await API.create(t, payer, data, 0, 0, handler);
  await createTx.assertSuccess(t);

  // mint 1 asset

  const amount = 1;

  const [masterEdition] = PublicKey.findProgramAddressSync(
    [Buffer.from('metadata'), PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Master Edition Account', masterEdition);
  const daManager = new DigitalAssetManager(mint, metadata, masterEdition);

  const { tx: mintTx, token } = await API.mint(
    t,
    connection,
    payer,
    mint,
    metadata,
    masterEdition,
    daManager.emptyAuthorizationData(),
    amount,
    handler,
  );
  await mintTx.assertSuccess(t);

  const tokenAccount = await getAccount(connection, token);

  spok(t, tokenAccount, {
    amount: spokSameBigint(new BN(1)),
    isFrozen: true,
    owner: payer.publicKey,
  });
});

test('Mint: ProgrammableNonFungible with existing token account', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    name: 'ProgrammableNonFungible',
    symbol: 'PNF',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
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

  const { tx: createTx, metadata, mint } = await API.create(t, payer, data, 0, 0, handler);
  await createTx.assertSuccess(t);

  // initialize a token account

  const { tx: tokenTx, token } = await API.createTokenAccount(
    mint,
    payer,
    connection,
    handler,
    payer.publicKey,
  );
  await tokenTx.assertSuccess(t);

  // mint 1 asset

  const amount = 1;

  const [masterEdition] = PublicKey.findProgramAddressSync(
    [Buffer.from('metadata'), PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Master Edition Account', masterEdition);
  const daManager = new DigitalAssetManager(mint, metadata, masterEdition);

  const { tx: mintTx } = await API.mint(
    t,
    connection,
    payer,
    mint,
    metadata,
    masterEdition,
    daManager.emptyAuthorizationData(),
    amount,
    handler,
    token,
  );
  await mintTx.assertSuccess(t);

  const tokenAccount = await getAccount(connection, token);

  spok(t, tokenAccount, {
    amount: spokSameBigint(new BN(1)),
    isFrozen: true,
    owner: payer.publicKey,
  });
});

test('Mint: fail to mint zero (0) tokens from ProgrammableNonFungible', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    name: 'ProgrammableNonFungible',
    symbol: 'PNF',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
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

  const { tx: createTx, metadata, mint } = await API.create(t, payer, data, 0, 0, handler);
  await createTx.assertSuccess(t);

  // mint 0 asset

  const [masterEdition] = PublicKey.findProgramAddressSync(
    [Buffer.from('metadata'), PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Master Edition Account', masterEdition);
  const daManager = new DigitalAssetManager(mint, metadata, masterEdition);

  const { tx: mintTx } = await API.mint(
    t,
    connection,
    payer,
    mint,
    metadata,
    masterEdition,
    daManager.emptyAuthorizationData(),
    0,
    handler,
  );
  await mintTx.assertError(t, /Amount must be greater than zero/);
});

test('Mint: fail to mint multiple from ProgrammableNonFungible', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    name: 'ProgrammableNonFungible',
    symbol: 'PNF',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
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

  const { tx: createTx, metadata, mint } = await API.create(t, payer, data, 0, 0, handler);
  await createTx.assertSuccess(t);

  // tries to mint 2 asset

  const [masterEdition] = PublicKey.findProgramAddressSync(
    [Buffer.from('metadata'), PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Master Edition Account', masterEdition);
  const manager = new DigitalAssetManager(mint, metadata, masterEdition);

  const { tx: multipleMintTx } = await API.mint(
    t,
    connection,
    payer,
    mint,
    metadata,
    masterEdition,
    manager.emptyAuthorizationData(),
    2,
    handler,
  );
  await multipleMintTx.assertError(t, /Editions must have exactly one token/);

  // tries to mint 1 asset

  const { tx: mintTx } = await API.mint(
    t,
    connection,
    payer,
    mint,
    metadata,
    masterEdition,
    manager.emptyAuthorizationData(),
    1,
    handler,
  );
  await mintTx.assertSuccess(t);

  // tries to mint another one asset

  const { tx: mintTx2 } = await API.mint(
    t,
    connection,
    payer,
    mint,
    metadata,
    masterEdition,
    manager.emptyAuthorizationData(),
    1,
    handler,
  );
  await mintTx2.assertError(t, /Editions must have exactly one token/);
});
