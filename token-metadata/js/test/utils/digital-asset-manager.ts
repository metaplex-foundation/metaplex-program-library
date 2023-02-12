import { Connection, PublicKey } from '@solana/web3.js';
import { AssetData, Metadata, TokenStandard, AuthorizationData } from '../../src/generated';
import { InitTransactions } from '../setup';
import test from 'tape';
import { PayerTransactionHandler } from '@metaplex-foundation/amman-client';
import { Keypair } from '@solana/web3.js';

export class DigitalAssetManager {
  mint: PublicKey;
  metadata: PublicKey;
  masterEdition: PublicKey;
  token?: PublicKey;

  constructor(mint: PublicKey, metadata: PublicKey, masterEdition: PublicKey) {
    this.mint = mint;
    this.metadata = metadata;
    this.masterEdition = masterEdition;
  }

  emptyAuthorizationData(): AuthorizationData {
    return {
      payload: {
        map: new Map(),
      },
    };
  }

  async getAssetData(connection: Connection): Promise<AssetData> {
    const md = await Metadata.fromAccountAddress(connection, this.metadata);

    return {
      name: md.data.name,
      symbol: md.data.symbol,
      uri: md.data.uri,
      sellerFeeBasisPoints: md.data.sellerFeeBasisPoints,
      creators: md.data.creators,
      primarySaleHappened: md.primarySaleHappened,
      isMutable: md.isMutable,
      tokenStandard: md.tokenStandard,
      collection: md.collection,
      uses: md.uses,
      collectionDetails: md.collectionDetails,
      ruleSet: md.programmableConfig ? md.programmableConfig.ruleSet : null,
    };
  }
}

export async function createDefaultAsset(
  t: test.Test,
  connection: Connection,
  API: InitTransactions,
  handler: PayerTransactionHandler,
  payer: Keypair,
  tokenStandard: TokenStandard = TokenStandard.NonFungible,
  ruleSet: PublicKey | null = null,
): Promise<DigitalAssetManager> {
  const name = 'DigitalAsset';
  const symbol = 'DA';
  const uri = 'uri';

  // Create the initial asset and ensure it was created successfully
  const assetData: AssetData = {
    name,
    symbol,
    uri,
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
    tokenStandard,
    collection: null,
    uses: null,
    collectionDetails: null,
    ruleSet,
  };

  const {
    tx: createTx,
    mint,
    metadata,
    masterEdition,
  } = await API.create(t, payer, assetData, 0, 0, handler);
  await createTx.assertSuccess(t);

  const daManager = new DigitalAssetManager(mint, metadata, masterEdition);

  return daManager;
}

export async function createAndMintDefaultAsset(
  t: test.Test,
  connection: Connection,
  API: InitTransactions,
  handler: PayerTransactionHandler,
  payer: Keypair,
  tokenStandard: TokenStandard = TokenStandard.NonFungible,
  ruleSet: PublicKey | null = null,
  amount = 1,
): Promise<DigitalAssetManager> {
  const daManager = await createDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    tokenStandard,
    ruleSet,
  );
  const { mint, metadata, masterEdition } = daManager;

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

  daManager.token = token;

  return daManager;
}
