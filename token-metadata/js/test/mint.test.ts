import { getAccount } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { BN } from 'bn.js';
import spok from 'spok';
import { AssetData, PROGRAM_ID, TokenStandard } from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { spokSameBigint } from './utils';
import { DigitalAssetManager } from './utils/DigitalAssetManager';

killStuckProcess();

test('Mint: ProgrammableNonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    updateAuthority: payer.publicKey,
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
    editionNonce: null,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    programmableConfig: null,
    delegateState: null,
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

test('Mint: ProgrammableNonFungible asset with existing token account', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    updateAuthority: payer.publicKey,
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
    editionNonce: null,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    programmableConfig: null,
    delegateState: null,
  };

  const { tx: createTx, metadata, mint } = await API.create(t, payer, data, 0, 0, handler);
  await createTx.assertSuccess(t);

  // initialize a token account

  const { tx: tokenTx, token } = await API.createTokenAccount(mint, payer, connection, handler);
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
