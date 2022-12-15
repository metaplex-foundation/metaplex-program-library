import { Connection, PublicKey } from '@solana/web3.js';
import { AssetData, DelegateState, Metadata } from 'src/generated';

export class DigitalAssetManager {
  mint: PublicKey;
  metadata: PublicKey;
  masterEdition: PublicKey;

  constructor(mint: PublicKey, metadata: PublicKey, masterEdition: PublicKey) {
    this.mint = mint;
    this.metadata = metadata;
    this.masterEdition = masterEdition;
  }

  async getAssetData(connection: Connection): Promise<AssetData> {
    const md = await Metadata.fromAccountAddress(connection, this.metadata);

    let delegateState: DelegateState | null = null;
    if (md.delegate != null) {
      delegateState = {
        __kind: 'Sale',
        fields: [md.delegate],
      };
    }

    return {
      name: md.data.name,
      symbol: md.data.symbol,
      uri: md.data.uri,
      sellerFeeBasisPoints: md.data.sellerFeeBasisPoints,
      updateAuthority: md.updateAuthority,
      creators: md.data.creators,
      primarySaleHappened: md.primarySaleHappened,
      isMutable: md.isMutable,
      editionNonce: md.editionNonce,
      tokenStandard: md.tokenStandard,
      collection: md.collection,
      uses: md.uses,
      collectionDetails: md.collectionDetails,
      programmableConfig: md.programmableConfig,
      delegateState,
    };
  }
}
