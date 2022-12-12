import { getAccount } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { BN } from 'bn.js';
import spok from 'spok';
import { AssetData, PROGRAM_ID, TokenStandard } from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { spokSameBigint } from './utils';

killStuckProcess();

test('Mint: ProgrammableNonFungible asset', async (t) => {
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

  const [masterEdition] = PublicKey.findProgramAddressSync(
    [Buffer.from('metadata'), PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Master Edition Account', masterEdition);

  const { tx: mintTx, token } = await API.mint(t, payer, mint, metadata, masterEdition, handler);
  await mintTx.assertSuccess(t);

  const tokenAccount = await getAccount(connection, token);

  spok(t, tokenAccount, {
    amount: spokSameBigint(new BN(1)),
    isFrozen: true,
    owner: payer.publicKey,
  });
});
