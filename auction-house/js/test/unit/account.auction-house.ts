import { AccountInfo, Keypair, PublicKey } from '@solana/web3.js';
import { AuctionHouseAccount } from 'src/accounts/AuctionHouse';
import { AuctionHouseAccountDataArgs } from 'src/generated/accounts';
import test from 'tape';
import spok from 'spok';

function quickKeypair(): [PublicKey, Uint8Array] {
  const kp = Keypair.generate();
  return [kp.publicKey, kp.secretKey];
}

test('account auction-house: round trip serilization', async (t) => {
  const [accountPubkey] = quickKeypair();
  const [creator] = quickKeypair();
  const [auctionHouseTreasury] = quickKeypair();
  const [treasuryWithdrawalDestination] = quickKeypair();
  const [feeWithdrawalDestination] = quickKeypair();
  const [treasuryMint] = quickKeypair();

  const args: AuctionHouseAccountDataArgs = {
    auctionHouseFeeAccount: creator,
    auctionHouseTreasury,
    treasuryWithdrawalDestination,
    feeWithdrawalDestination,
    treasuryMint,
    authority: creator,
    creator,
    bump: 0,
    treasuryBump: 1,
    feePayerBump: 2,
    sellerFeeBasisPoints: 3,
    requiresSignOff: false,
    canChangeSalePrice: true,
  };

  const expected = AuctionHouseAccount.fromAccountArgs(accountPubkey, args);
  const [data] = expected.data.serialize();

  const info: AccountInfo<Buffer> = {
    executable: false,
    data,
    owner: creator,
    lamports: 1000,
  };

  const actual = AuctionHouseAccount.fromAccountInfo(accountPubkey, info);
  spok(t, actual, expected);
});
