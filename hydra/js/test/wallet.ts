import { Account, Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { NodeWallet } from '@project-serum/common'; //TODO remove this
import { NATIVE_MINT, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { expect, use } from 'chai';
import ChaiAsPromised from 'chai-as-promised';
import { Fanout, FanoutClient, FanoutMembershipVoucher, FanoutMint, MembershipModel } from '../src';
import { LOCALHOST } from '@metaplex-foundation/amman';
import { builtWalletFanout } from './utils/scenarios';

use(ChaiAsPromised);

describe('fanout', async () => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const lamportsNeeded = 10000000000;
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

  describe('Wallet membership model', () => {
    it('Init', async () => {
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 100,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Wallet,
      });

      const fanoutAccount = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
      expect(fanoutAccount.membershipModel).to.equal(MembershipModel.Wallet);
      expect(fanoutAccount.lastSnapshotAmount.toString()).to.equal('0');
      expect(fanoutAccount.totalMembers.toString()).to.equal('0');
      expect(fanoutAccount.totalInflow.toString()).to.equal('0');
      expect(fanoutAccount.totalAvailableShares.toString()).to.equal('100');
      expect(fanoutAccount.totalShares.toString()).to.equal('100');
      expect(fanoutAccount.membershipMint).to.equal(null);
      expect(fanoutAccount.totalStakedShares).to.equal(null);
    });

    it('Init For mint', async () => {
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 100,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Wallet,
      });
      const mint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const { fanoutForMint, tokenAccount } = await fanoutSdk.initializeFanoutForMint({
        fanout,
        mint: mint.publicKey,
      });

      const fanoutMintAccount = await fanoutSdk.fetch<FanoutMint>(fanoutForMint, FanoutMint);

      expect(fanoutMintAccount.mint.toBase58()).to.equal(mint.publicKey.toBase58());
      expect(fanoutMintAccount.fanout.toBase58()).to.equal(fanout.toBase58());
      expect(fanoutMintAccount.tokenAccount.toBase58()).to.equal(tokenAccount.toBase58());
      expect(fanoutMintAccount.totalInflow.toString()).to.equal('0');
      expect(fanoutMintAccount.lastSnapshotAmount.toString()).to.equal('0');
    });

    it('Init For Wrapped Sol', async () => {
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 100,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Wallet,
      });

      const { fanoutForMint, tokenAccount } = await fanoutSdk.initializeFanoutForMint({
        fanout,
        mint: NATIVE_MINT,
      });

      const fanoutMintAccount = await fanoutSdk.fetch<FanoutMint>(fanoutForMint, FanoutMint);

      expect(fanoutMintAccount.mint.toBase58()).to.equal(NATIVE_MINT.toBase58());
      expect(fanoutMintAccount.fanout.toBase58()).to.equal(fanout.toBase58());
      expect(fanoutMintAccount.tokenAccount.toBase58()).to.equal(tokenAccount.toBase58());
      expect(fanoutMintAccount.totalInflow.toString()).to.equal('0');
      expect(fanoutMintAccount.lastSnapshotAmount.toString()).to.equal('0');
    });

    it('Adds Members With Wallet', async () => {
      const init = await fanoutSdk.initializeFanout({
        totalShares: 100,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Wallet,
      });
      const member = new Keypair();
      const { membershipAccount } = await fanoutSdk.addMemberWallet({
        fanout: init.fanout,
        fanoutNativeAccount: init.nativeAccount,
        membershipKey: member.publicKey,
        shares: 10,
      });
      const fanoutAccount = await fanoutSdk.fetch<Fanout>(init.fanout, Fanout);
      const membershipAccountData = await fanoutSdk.fetch<FanoutMembershipVoucher>(
        membershipAccount,
        FanoutMembershipVoucher,
      );
      expect(fanoutAccount.membershipModel).to.equal(MembershipModel.Wallet);
      expect(fanoutAccount.lastSnapshotAmount.toString()).to.equal('0');
      expect(fanoutAccount.totalMembers.toString()).to.equal('1');
      expect(fanoutAccount.totalInflow.toString()).to.equal('0');
      expect(fanoutAccount.totalAvailableShares.toString()).to.equal('90');
      expect(fanoutAccount.totalShares.toString()).to.equal('100');
      expect(fanoutAccount.membershipMint).to.equal(null);
      expect(fanoutAccount.totalStakedShares).to.equal(null);
      expect(membershipAccountData?.shares?.toString()).to.equal('10');
      expect(membershipAccountData?.membershipKey?.toBase58()).to.equal(
        member.publicKey.toBase58(),
      );
    });

    it('Distribute a Native Fanout with Wallet Members', async () => {
      const builtFanout = await builtWalletFanout(fanoutSdk, 100, 5);
      expect(builtFanout.fanoutAccountData.totalAvailableShares.toString()).to.equal('0');
      expect(builtFanout.fanoutAccountData.totalMembers.toString()).to.equal('5');
      expect(builtFanout.fanoutAccountData.lastSnapshotAmount.toString()).to.equal('0');
      const distBot = new Keypair();
      await connection.requestAirdrop(builtFanout.fanoutAccountData.accountKey, lamportsNeeded);
      await connection.requestAirdrop(distBot.publicKey, lamportsNeeded);

      const member1 = builtFanout.members[0];
      const member2 = builtFanout.members[1];
      const distMember1 = await fanoutSdk.distributeWalletMemberInstructions({
        distributeForMint: false,
        member: member1.wallet.publicKey,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
      });
      const distMember2 = await fanoutSdk.distributeWalletMemberInstructions({
        distributeForMint: false,
        member: member2.wallet.publicKey,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
      });
      const holdingAccountReserved = await connection.getMinimumBalanceForRentExemption(1);
      const memberDataBefore1 = await connection.getAccountInfo(member1.wallet.publicKey);
      const memberDataBefore2 = await connection.getAccountInfo(member2.wallet.publicKey);
      const holdingAccountBefore = await connection.getAccountInfo(
        builtFanout.fanoutAccountData.accountKey,
      );
      expect(memberDataBefore2).to.be.null;
      expect(memberDataBefore1).to.be.null;
      const firstSnapshot = lamportsNeeded;
      expect(holdingAccountBefore?.lamports + lamportsNeeded).to.equal(
        firstSnapshot + holdingAccountReserved,
      );
      const tx = await fanoutSdk.sendInstructions(
        [...distMember1.instructions, ...distMember2.instructions],
        [distBot],
        distBot.publicKey,
      );
      if (!!tx.RpcResponseAndContext.value.err) {
        const txdetails = await connection.getConfirmedTransaction(tx.TransactionSignature);
        console.log(txdetails, tx.RpcResponseAndContext.value.err);
      }
      const memberDataAfter1 = await connection.getAccountInfo(member1.wallet.publicKey);
      const memberDataAfter2 = await connection.getAccountInfo(member2.wallet.publicKey);
      const holdingAccountAfter = await connection.getAccountInfo(
        builtFanout.fanoutAccountData.accountKey,
      );
      const membershipAccount1 = await fanoutSdk.fetch<FanoutMembershipVoucher>(
        member1.voucher,
        FanoutMembershipVoucher,
      );

      expect(memberDataAfter1?.lamports).to.equal(firstSnapshot * 0.2);
      expect(memberDataAfter2?.lamports).to.equal(firstSnapshot * 0.2);
      expect(holdingAccountAfter?.lamports).to.equal(
        firstSnapshot - firstSnapshot * 0.4 + holdingAccountReserved,
      );
      expect(builtFanout.fanoutAccountData.lastSnapshotAmount.toString()).to.equal('0');
      expect(membershipAccount1.totalInflow.toString()).to.equal(`${firstSnapshot * 0.2}`);
    });

    it('Transfer Shares', async () => {
      const builtFanout = await builtWalletFanout(fanoutSdk, 100, 5);
      const sent = 10;
      await connection.requestAirdrop(builtFanout.fanoutAccountData.accountKey, sent);
      await connection.requestAirdrop(fanoutSdk.wallet.publicKey, 1);
      const member0Wallet = builtFanout.members[0].wallet;
      const member1Wallet = builtFanout.members[1].wallet;
      const member0Voucher = builtFanout.members[0].voucher;
      const member1Voucher = builtFanout.members[1].voucher;

      await fanoutSdk.transferShares({
        fromMember: member0Wallet.publicKey,
        toMember: member1Wallet.publicKey,
        fanout: builtFanout.fanout,
        shares: 20,
      });

      const membershipAccount0 = await fanoutSdk.fetch<FanoutMembershipVoucher>(
        member0Voucher,
        FanoutMembershipVoucher,
      );
      const membershipAccount1 = await fanoutSdk.fetch<FanoutMembershipVoucher>(
        member1Voucher,
        FanoutMembershipVoucher,
      );

      expect(membershipAccount0.shares.toString()).to.equal('0');
      expect(membershipAccount1.shares.toString()).to.equal('40');
    });

    it('Remove Member', async () => {
      const builtFanout = await builtWalletFanout(fanoutSdk, 100, 5);
      const sent = 10;
      const rando = new Keypair();
      await connection.requestAirdrop(builtFanout.fanoutAccountData.accountKey, sent);
      await connection.requestAirdrop(fanoutSdk.wallet.publicKey, 1);
      const member0Wallet = builtFanout.members[0].wallet;
      const member1Wallet = builtFanout.members[1].wallet;
      const member0Voucher = builtFanout.members[0].voucher;

      await fanoutSdk.transferShares({
        fromMember: member0Wallet.publicKey,
        toMember: member1Wallet.publicKey,
        fanout: builtFanout.fanout,
        shares: 20,
      });
      await fanoutSdk.removeMember({
        destination: rando.publicKey,
        fanout: builtFanout.fanout,
        member: member0Wallet.publicKey,
      });

      const fanout_after = await fanoutSdk.fetch<Fanout>(builtFanout.fanout, Fanout);
      expect(fanout_after.totalMembers.toString()).to.equal('4');

      expect(fanoutSdk.getAccountInfo(member0Voucher)).to.be.rejectedWith(
        new Error('Account Not Found'),
      );
    });
  });
});
