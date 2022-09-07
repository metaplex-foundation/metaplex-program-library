import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { Metaplex, keypairIdentity } from '@metaplex-foundation/js'
import { COLLECTION_METADATA } from '../../../candy-core/js/test/utils';

const API = new InitTransactions();

killStuckProcess();

test('nft payment (missing accounts)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const metaplex = Metaplex.make(connection)
        .use(keypairIdentity(payerPair));

    const { nft: collection } = await metaplex
        .nfts()
        .create({
            uri: COLLECTION_METADATA,
            name: 'CORE Collection',
            sellerFeeBasisPoints: 500,
        })
        .run();

  const data = {
    default: {
      botTax: null,
      liveDate: {
        date: 1662479807,
      },
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: null,
    },
    groups: null,
  };

  const { candyGuard, candyMachine } = await API.deploy(
    t,
    data,
    payerPair,
    fstTxHandler,
    connection,
    collection.address,
  );

  // mint (as an authority)

  const [, mintForAuthority] = await amman.genLabeledKeypair('Mint Account (authority)');
  const { tx: authorityMintTx } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    payerPair,
    mintForAuthority,
    fstTxHandler,
    connection
  );
  await authorityMintTx.assertError(t, /Missing collection accounts/i);

  // mint (as a minter)

  const { fstTxHandler: minterHandler, minterPair: minter, connection: minterConnection } = await API.minter();
  const [, mintForMinter] = await amman.genLabeledKeypair('Mint Account (minter)');
  const { tx: minterMintTx } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minter,
    mintForMinter,
    minterHandler,
    minterConnection
  );
  await minterMintTx.assertError(t, /Missing collection accounts/i);

  /*
  const data = {
    default: {
      botTax: null,
      liveDate: {
        date: 1662479807,
      },
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: {
        burn: true,
        requiredCollection: collection.address,
      },
    },
    groups: null,
  };*/
});
