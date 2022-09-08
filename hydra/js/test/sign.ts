/* eslint-disable @typescript-eslint/no-unused-vars */
import { Account, Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { NodeWallet } from '@project-serum/common'; //TODO remove this
import { expect, use } from 'chai';
import ChaiAsPromised from 'chai-as-promised';
import { Fanout, FanoutClient, MembershipModel } from '../src';
import { createMasterEdition } from './utils/metaplex';
import { deprecated } from '@metaplex-foundation/mpl-token-metadata';
import { LOCALHOST } from '@metaplex-foundation/amman';

use(ChaiAsPromised);

describe('fanout', async () => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  let authorityWallet: Keypair;
  let fanoutSdk: FanoutClient;
  beforeEach(async () => {
    authorityWallet = Keypair.generate();
    await connection.requestAirdrop(authorityWallet.publicKey, LAMPORTS_PER_SOL * 10);
    fanoutSdk = new FanoutClient(
      connection,
      new NodeWallet(new Account(authorityWallet.secretKey)),
    );
    await connection.requestAirdrop(authorityWallet.publicKey, LAMPORTS_PER_SOL * 10);
  });

  describe('NFT Signing', () => {
    it('Can Sign As Creator', async () => {
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 100,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.NFT,
      });

      const fanoutAccount = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
      const initMetadataData = new deprecated.DataV2({
        uri: 'URI',
        name: 'NAME',
        symbol: 'SYMBOL',
        sellerFeeBasisPoints: 1000,
        creators: [
          new deprecated.Creator({
            address: authorityWallet.publicKey.toBase58(),
            share: 0,
            verified: true,
          }),
          new deprecated.Creator({
            address: fanoutAccount.accountKey.toBase58(),
            share: 100,
            verified: false,
          }),
        ],
        collection: null,
        uses: null,
      });
      const nft = await createMasterEdition(
        connection,
        authorityWallet,
        //@ts-ignore
        initMetadataData,
        0,
      );

      //@ts-ignore
      const sign = await fanoutSdk.signMetadata({
        fanout: fanout,
        metadata: nft.metadata,
      });

      const meta = await deprecated.Metadata.findByMint(connection, nft.mint.publicKey);
      expect(meta?.data?.data?.creators?.at(1)?.verified).to.equal(1);
      expect(meta?.data?.data?.creators?.at(1)?.address).to.equal(
        fanoutAccount.accountKey.toBase58(),
      );
    });
  });
});
