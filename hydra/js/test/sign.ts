/* eslint-disable @typescript-eslint/no-unused-vars */
import { Account, Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { NodeWallet } from '@project-serum/common'; //TODO remove this
import { expect, use } from 'chai';
import ChaiAsPromised from 'chai-as-promised';
import { Fanout, FanoutClient, MembershipModel } from '../src';
import { keypairIdentity, Metaplex } from '@metaplex-foundation/js';
import { LOCALHOST } from '@metaplex-foundation/amman';

use(ChaiAsPromised);

describe('fanout', async () => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const metaplex = new Metaplex(connection);
  let authorityWallet: Keypair;
  metaplex.use(keypairIdentity(authorityWallet));
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
      const { nft } = await metaplex.nfts().create({
        uri: 'URI',
        name: 'NAME',
        symbol: 'SYMBOL',
        sellerFeeBasisPoints: 1000,
        creators: [
          {
            address: authorityWallet.publicKey,
            share: 0,
            authority: authorityWallet,
          },
          {
            address: fanoutAccount.accountKey,
            share: 100,
          },
        ],
      });

      //@ts-ignore
      const sign = await fanoutSdk.signMetadata({
        fanout: fanout,
        metadata: nft.metadataAddress,
      });

      const meta = await metaplex.nfts().findByMint({ mintAddress: nft.mint.address });
      expect(meta.creators.at(1).verified);
      expect(meta.creators.at(1).address).to.equal(fanoutAccount.accountKey.toBase58());
    });
  });
});
