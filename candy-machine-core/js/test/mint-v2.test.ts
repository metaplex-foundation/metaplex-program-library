import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import spok from 'spok';
import { AccountVersion, CandyMachine, CandyMachineData, ConfigLine } from '../src/generated';
import { TokenStandard } from '@metaplex-foundation/mpl-token-metadata';
import { keypairIdentity, Metaplex } from '@metaplex-foundation/js';
import { getAccount } from '@solana/spl-token';
import { spokSamePubkey } from './utils';

killStuckProcess();

test('mintV2: Programmable NFT', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 10;

  const data: CandyMachineData = {
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

  const { tx: transaction, candyMachine: address } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  const lines: ConfigLine[] = [];

  for (let i = 0; i < items; i++) {
    lines[i] = {
      name: `pNFT #${i + 1}`,
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }

  const { txs } = await API.addConfigLines(t, address, payerPair, lines, 0);
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertSuccess(t);
  }

  // to pNFT
  const candyMachineObject = await CandyMachine.fromAccountAddress(connection, address);

  const { tx: txpNft } = await API.setTokenStandard(
    t,
    payerPair,
    address,
    candyMachineObject.collectionMint,
    payerPair,
    TokenStandard.ProgrammableNonFungible,
    fstTxHandler,
    connection,
  );
  await txpNft.assertSuccess(t);

  const { tx: mintTransaction, mintAddress: mint } = await API.mintV2(
    t,
    address,
    payerPair,
    fstTxHandler,
    connection,
  );
  await mintTransaction.assertSuccess(t);

  const metaplex = Metaplex.make(connection).use(keypairIdentity(payerPair));
  const nftTokenAccount = metaplex
    .tokens()
    .pdas()
    .associatedTokenAccount({ mint: mint, owner: payerPair.publicKey });

  const ataAccount = await getAccount(connection, nftTokenAccount);

  spok(t, ataAccount, {
    isFrozen: true,
    mint: spokSamePubkey(mint),
  });

  const nft = await metaplex.nfts().findByMint({ mintAddress: mint });

  spok(t, nft, {
    mint: {
      address: spokSamePubkey(mint),
    },
    tokenStandard: TokenStandard.ProgrammableNonFungible,
  });
});

test('mintV2: NFT', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 10;

  const data: CandyMachineData = {
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

  const { tx: transaction, candyMachine: address } = await API.initializeV2(
    t,
    payerPair,
    data,
    TokenStandard.NonFungible,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  const lines: ConfigLine[] = [];

  for (let i = 0; i < items; i++) {
    lines[i] = {
      name: `NFT #${i + 1}`,
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }

  const { txs } = await API.addConfigLines(t, address, payerPair, lines, 0);
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertSuccess(t);
  }

  const { tx: mintTransaction, mintAddress: mint } = await API.mintV2(
    t,
    address,
    payerPair,
    fstTxHandler,
    connection,
  );
  await mintTransaction.assertSuccess(t);

  const metaplex = Metaplex.make(connection).use(keypairIdentity(payerPair));
  const nftTokenAccount = metaplex
    .tokens()
    .pdas()
    .associatedTokenAccount({ mint: mint, owner: payerPair.publicKey });

  const ataAccount = await getAccount(connection, nftTokenAccount);

  spok(t, ataAccount, {
    isFrozen: false,
    mint: spokSamePubkey(mint),
  });

  const nft = await metaplex.nfts().findByMint({ mintAddress: mint });

  spok(t, nft, {
    mint: {
      address: spokSamePubkey(mint),
    },
    tokenStandard: TokenStandard.NonFungible,
  });
});

test('mintV2: mint from existing candy machine', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 10;

  const data: CandyMachineData = {
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

  const { tx: transaction, candyMachine: address } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  const lines: ConfigLine[] = [];

  for (let i = 0; i < items; i++) {
    lines[i] = {
      name: `NFT #${i + 1}`,
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }

  const { txs } = await API.addConfigLines(t, address, payerPair, lines, 0);
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertSuccess(t);
  }

  const { tx: mintTransaction, mintAddress: nftMint } = await API.mint(
    t,
    address,
    payerPair,
    fstTxHandler,
    connection,
  );
  await mintTransaction.assertSuccess(t);

  const metaplex = Metaplex.make(connection).use(keypairIdentity(payerPair));
  const nft = await metaplex.nfts().findByMint({ mintAddress: nftMint });

  spok(t, nft, {
    mint: {
      address: spokSamePubkey(nftMint),
    },
    tokenStandard: TokenStandard.NonFungible,
  });

  // set token standard to pNFT

  let candyMachine = await CandyMachine.fromAccountAddress(connection, address);

  const { tx: txNFT } = await API.setTokenStandard(
    t,
    payerPair,
    address,
    candyMachine.collectionMint,
    payerPair,
    TokenStandard.ProgrammableNonFungible,
    fstTxHandler,
    connection,
  );
  await txNFT.assertSuccess(t);

  candyMachine = await CandyMachine.fromAccountAddress(connection, address);
  spok(t, candyMachine, {
    version: AccountVersion.V2,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
  });

  // mints a pNFT

  const { tx: mintTransaction2, mintAddress: pnftMint } = await API.mintV2(
    t,
    address,
    payerPair,
    fstTxHandler,
    connection,
  );
  await mintTransaction2.assertSuccess(t);

  const pnft = await metaplex.nfts().findByMint({ mintAddress: pnftMint });

  spok(t, pnft, {
    mint: {
      address: spokSamePubkey(pnftMint),
    },
    tokenStandard: TokenStandard.ProgrammableNonFungible,
  });
});
