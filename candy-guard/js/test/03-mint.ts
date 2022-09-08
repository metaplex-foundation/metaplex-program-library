import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { CandyMachineHelper } from './utils';

const API = new InitTransactions();
const HELPER = new CandyMachineHelper();

killStuckProcess();

test('mint (CPI)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  // candy machine
  const [, candyMachine] = await amman.genLabeledKeypair('Candy Machine Account');

  const items = 10;

  const candyMachineData = {
    itemsAvailable: items,
    symbol: 'CORE',
    sellerFeeBasisPoints: 500,
    maxSupply: 0,
    isMutable: true,
    creators: [
      {
        address: payerPair.publicKey,
        verified: false,
        percentageShare: 100,
      },
    ],
    configLineSettings: {
      prefixName: 'TEST ',
      nameLength: 10,
      prefixUri: 'https://arweave.net/',
      uriLength: 50,
      isSequential: false,
    },
    hiddenSettings: null,
  };

  const { tx: createTxCM } = await HELPER.create(
    t,
    payerPair,
    candyMachine,
    candyMachineData,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await createTxCM.assertNone();

  const lines: { name: string; uri: string }[] = [];

  for (let i = 0; i < items; i++) {
    const line = {
      name: `NFT #${i + 1}`,
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };

    lines.push(line);
  }
  const { txs } = await HELPER.addConfigLines(
    t,
    candyMachine.publicKey,
    payerPair,
    lines,
    fstTxHandler,
  );
  // confirms that all lines have been written
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertNone();
  }

  // minting directly from the candy machine

  // as authority
  const [, mintKeypair1] = await amman.genLabeledKeypair('Mint Account (authority)');
  const { tx: mintTx1 } = await HELPER.mint(
    t,
    candyMachine.publicKey,
    payerPair,
    mintKeypair1,
    fstTxHandler,
    connection,
  );
  await mintTx1.assertSuccess(t);

  // as a minter
  try {
    const {
      fstTxHandler: minterHandler,
      minterPair: minter,
      connection: minterConnection,
    } = await API.minter();
    const [, mintKeypair2] = await amman.genLabeledKeypair('Mint Account (minter)');
    const { tx: mintTx2 } = await HELPER.mint(
      t,
      candyMachine.publicKey,
      minter,
      mintKeypair2,
      minterHandler,
      minterConnection,
    );
    await mintTx2.assertSuccess(t);
    t.fail('only authority is allowed to mint');
  } catch {
    // we are expecting an error
    t.pass('minter is not the candy machine authority');
  }

  // candy guard
  const candyGuardData = {
    default: {
      botTax: null,
      lamports: null,
      splToken: null,
      liveDate: null,
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

  const { tx: initializeTxCG, candyGuard: address } = await API.initialize(
    t,
    candyGuardData,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await initializeTxCG.assertSuccess(t);

  const { tx: wrapTx } = await API.wrap(
    t,
    address,
    candyMachine.publicKey,
    payerPair,
    fstTxHandler,
  );

  await wrapTx.assertSuccess(t, [/SetAuthority/i]);

  // minting from the candy machie should fail

  try {
    const [, mintKeypair3] = await amman.genLabeledKeypair('CG Mint Account (authority)');
    const { tx: mintTx3 } = await HELPER.mint(
      t,
      candyMachine.publicKey,
      payerPair,
      mintKeypair3,
      fstTxHandler,
      connection,
    );
    await mintTx3.assertSuccess(t);
    t.fail('only CG authority is allowed to mint');
  } catch {
    // we are expecting an error
    t.pass('payer is not the candy machine authority');
  }

  // minting through the candy guard (as authority)

  const [, mintKeypair4] = await amman.genLabeledKeypair('CG Mint Account (authority)');
  const { tx: mintTx4 } = await API.mint(
    t,
    address,
    candyMachine.publicKey,
    payerPair,
    mintKeypair4,
    fstTxHandler,
    connection,
  );
  await mintTx4.assertSuccess(t);

  // minting through the candy guard (as a minter)

  const {
    fstTxHandler: minterHandler,
    minterPair: minter,
    connection: minterConnection,
  } = await API.minter();
  const [, mintKeypair5] = await amman.genLabeledKeypair('CG Mint Account (minter)');
  const { tx: mintTx5 } = await API.mint(
    t,
    address,
    candyMachine.publicKey,
    minter,
    mintKeypair5,
    minterHandler,
    minterConnection,
  );
  await mintTx5.assertSuccess(t);
});
