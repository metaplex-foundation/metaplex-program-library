import spok from 'spok';
import { AssetData, Metadata, TokenStandard } from '../src/generated';
import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';

killStuckProcess();

test('Create: ProgrammableNonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const data: AssetData = {
    name: 'ProgrammableNonFungible',
    symbol: 'PNF',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
    updateAuthority: payer.publicKey,
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

  const { tx: transaction, metadata: address } = await API.create(t, payer, data, 0, 0, handler);
  // executes the transaction
  await transaction.assertSuccess(t);

  const metadata = await Metadata.fromAccountAddress(connection, address);

  spok(t, metadata, {
    data: {
      sellerFeeBasisPoints: 0,
    },
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
  });

  t.equal(metadata.data.name.replace(/\0+/, ''), 'ProgrammableNonFungible');
  t.equal(metadata.data.symbol.replace(/\0+/, ''), 'PNF');
  t.equal(metadata.data.uri.replace(/\0+/, ''), 'uri');
});

test('Create: ProgrammableNonFungible with existing mint account', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  // initialize a mint account

  const { tx: mintTx, mint } = await API.createMintAccount(payer, connection, handler);
  await mintTx.assertSuccess(t);

  // create the metadata

  const data: AssetData = {
    name: 'ProgrammableNonFungible',
    symbol: 'PNF',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
    updateAuthority: payer.publicKey,
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

  const { tx: transaction, metadata: address } = await API.create(
    t,
    payer,
    data,
    0,
    0,
    handler,
    mint,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  const metadata = await Metadata.fromAccountAddress(connection, address);

  spok(t, metadata, {
    data: {
      sellerFeeBasisPoints: 0,
    },
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
  });

  t.equal(metadata.data.name.replace(/\0+/, ''), 'ProgrammableNonFungible');
  t.equal(metadata.data.symbol.replace(/\0+/, ''), 'PNF');
  t.equal(metadata.data.uri.replace(/\0+/, ''), 'uri');
});
