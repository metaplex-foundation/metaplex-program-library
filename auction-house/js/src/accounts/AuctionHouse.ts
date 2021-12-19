// This will wrap the AuctionHouse account to include various checks
// The generated account mainly serves to create/serialize/deserialize

import { AccountInfo, PublicKey } from '@solana/web3.js';
import { AuctionHouseAccountData } from '../generated/accounts';

export class AuctionHouseAccount {
  readonly data: AuctionHouseAccountData;
  constructor(_pubkey: PublicKey, info: AccountInfo<Buffer>) {
    // TODO(thlorenz): assert data compatibility

    [this.data] = AuctionHouseAccountData.fromAccountInfo(info);
  }
}
