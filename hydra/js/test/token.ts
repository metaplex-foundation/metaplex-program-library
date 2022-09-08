/* eslint-disable @typescript-eslint/no-unused-vars */
import { Account, Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { NodeWallet } from '@project-serum/common'; //TODO remove this
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { expect, use } from 'chai';
import ChaiAsPromised from 'chai-as-promised';
import { Fanout, FanoutClient, FanoutMembershipVoucher, FanoutMint, MembershipModel } from '../src';
import { LOCALHOST } from '@metaplex-foundation/amman';
import { builtTokenFanout } from './utils/scenarios';
import BN from 'bn.js';

use(ChaiAsPromised);

describe('fanout', async () => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const lamportsNeeded = 10000000000;
  let authorityWallet: Keypair;
  let fanoutSdk: FanoutClient;
  beforeEach(async () => {
    authorityWallet = Keypair.generate();
    let signature = await connection.requestAirdrop(authorityWallet.publicKey, lamportsNeeded);
    await connection.confirmTransaction(signature);
    fanoutSdk = new FanoutClient(
      connection,
      new NodeWallet(new Account(authorityWallet.secretKey)),
    );
    signature = await connection.requestAirdrop(authorityWallet.publicKey, lamportsNeeded);
    await connection.confirmTransaction(signature);
  });

  describe('Token membership model', () => {
    it('Creates fanout w/ token, 2 members stake, has 5 random revenue events, and distributes', async () => {
      const membershipMint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const distBot = new Keypair();
      await connection.requestAirdrop(distBot.publicKey, lamportsNeeded);
      const supply = 1000000 * 10 ** 6;
      const tokenAcct = await membershipMint.createAccount(authorityWallet.publicKey);
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint.publicKey,
      });
      const mint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const mintAcctAuthority = await mint.createAssociatedTokenAccount(authorityWallet.publicKey);
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
      let totalStaked = 0;
      const members = [];
      await membershipMint.mintTo(tokenAcct, authorityWallet.publicKey, [], supply);
      for (let index = 0; index <= 4; index++) {
        const member = new Keypair();
        const pseudoRng = Math.floor(supply * Math.random() * 0.138);
        await connection.requestAirdrop(member.publicKey, lamportsNeeded);
        const tokenAcctMember = await membershipMint.createAssociatedTokenAccount(member.publicKey);
        const mintAcctMember = await mint.createAssociatedTokenAccount(member.publicKey);
        await membershipMint.transfer(
          tokenAcct,
          tokenAcctMember,
          authorityWallet.publicKey,
          [],
          pseudoRng,
        );
        totalStaked += pseudoRng;
        const ixs = await fanoutSdk.stakeTokenMemberInstructions({
          shares: pseudoRng,
          fanout: fanout,
          membershipMintTokenAccount: tokenAcctMember,
          membershipMint: membershipMint.publicKey,
          member: member.publicKey,
          payer: member.publicKey,
        });
        const tx = await fanoutSdk.sendInstructions(ixs.instructions, [member], member.publicKey);
        if (!!tx.RpcResponseAndContext.value.err) {
          const txdetails = await connection.getConfirmedTransaction(tx.TransactionSignature);
          console.log(txdetails, tx.RpcResponseAndContext.value.err);
        }
        const voucher = await fanoutSdk.fetch<FanoutMembershipVoucher>(
          ixs.output.membershipVoucher,
          FanoutMembershipVoucher,
        );

        expect(voucher.shares?.toString()).to.equal(`${pseudoRng}`);
        expect(voucher.membershipKey?.toBase58()).to.equal(member.publicKey.toBase58());
        expect(voucher.fanout?.toBase58()).to.equal(fanout.toBase58());
        const stake = await membershipMint.getAccountInfo(ixs.output.stakeAccount);
        expect(stake.amount.toString()).to.equal(`${pseudoRng}`);
        members.push({
          member,
          membershipTokenAccount: tokenAcctMember,
          fanoutMintTokenAccount: mintAcctMember,
          shares: pseudoRng,
        });
      }
      //@ts-ignore
      let runningTotal = 0;
      for (let index = 0; index <= 4; index++) {
        const sent = Math.floor(Math.random() * 100 * 10 ** 6);
        await mint.mintTo(mintAcctAuthority, authorityWallet.publicKey, [], sent);
        await mint.transfer(mintAcctAuthority, tokenAccount, authorityWallet.publicKey, [], sent);
        runningTotal += sent;
        const member = members[index];
        const ix = await fanoutSdk.distributeTokenMemberInstructions({
          distributeForMint: true,
          fanoutMint: mint.publicKey,
          membershipMint: membershipMint.publicKey,
          fanout: fanout,
          member: member.member.publicKey,
          payer: distBot.publicKey,
        });
        // @ts-ignore
        const tx = await fanoutSdk.sendInstructions(ix.instructions, [distBot], distBot.publicKey);

        if (!!tx.RpcResponseAndContext.value.err) {
          const txdetails = await connection.getConfirmedTransaction(tx.TransactionSignature);
          console.log(txdetails, tx.RpcResponseAndContext.value.err);
        }
        const tokenAcctInfo = await connection.getTokenAccountBalance(
          member.fanoutMintTokenAccount,
          'confirmed',
        );
        const diff = ((supply - totalStaked) * sent) / totalStaked;
        const amountDist = (member.shares * diff) / supply;
        expect(tokenAcctInfo.value.amount, `${amountDist}`);
        // @ts-ignore
      }
    });

    it('Init', async () => {
      const membershipMint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const supply = 1000000 * 10 ** 6;
      const tokenAcct = await membershipMint.createAccount(authorityWallet.publicKey);
      await membershipMint.mintTo(tokenAcct, authorityWallet.publicKey, [], supply);
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint.publicKey,
      });

      const fanoutAccount = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
      expect(fanoutAccount.membershipModel).to.equal(MembershipModel.Token);
      expect(fanoutAccount.lastSnapshotAmount.toString()).to.equal('0');
      expect(fanoutAccount.totalMembers.toString()).to.equal('0');
      expect(fanoutAccount.totalInflow.toString()).to.equal('0');
      expect(fanoutAccount.totalAvailableShares.toString()).to.equal('0');
      expect(fanoutAccount.totalShares.toString()).to.equal(supply.toString());
      expect(fanoutAccount.membershipMint?.toBase58()).to.equal(
        membershipMint.publicKey.toBase58(),
      );
      expect(fanoutAccount.totalStakedShares?.toString()).to.equal('0');
    });

    it('Init For mint', async () => {
      const membershipMint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const supply = 1000000 * 10 ** 6;
      const tokenAcct = await membershipMint.createAccount(authorityWallet.publicKey);
      await membershipMint.mintTo(tokenAcct, authorityWallet.publicKey, [], supply);
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint.publicKey,
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

    it('Stakes Members', async () => {
      const membershipMint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const supply = 1000000 * 10 ** 6;
      const member = new Keypair();
      await connection.requestAirdrop(member.publicKey, lamportsNeeded);
      const tokenAcct = await membershipMint.createAccount(authorityWallet.publicKey);
      const tokenAcctMember = await membershipMint.createAssociatedTokenAccount(member.publicKey);
      await membershipMint.mintTo(tokenAcct, authorityWallet.publicKey, [], supply);
      await membershipMint.transfer(
        tokenAcct,
        tokenAcctMember,
        authorityWallet.publicKey,
        [],
        supply * 0.1,
      );

      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint.publicKey,
      });
      const ixs = await fanoutSdk.stakeTokenMemberInstructions({
        shares: supply * 0.1,
        fanout: fanout,
        membershipMintTokenAccount: tokenAcctMember,
        membershipMint: membershipMint.publicKey,
        member: member.publicKey,
        payer: member.publicKey,
      });
      const tx = await fanoutSdk.sendInstructions(ixs.instructions, [member], member.publicKey);
      if (!!tx.RpcResponseAndContext.value.err) {
        const txdetails = await connection.getConfirmedTransaction(tx.TransactionSignature);
        console.log(txdetails, tx.RpcResponseAndContext.value.err);
      }
      const voucher = await fanoutSdk.fetch<FanoutMembershipVoucher>(
        ixs.output.membershipVoucher,
        FanoutMembershipVoucher,
      );

      expect(voucher.shares?.toString()).to.equal(`${supply * 0.1}`);
      expect(voucher.membershipKey?.toBase58()).to.equal(member.publicKey.toBase58());
      expect(voucher.fanout?.toBase58()).to.equal(fanout.toBase58());
      const stake = await membershipMint.getAccountInfo(ixs.output.stakeAccount);
      expect(stake.amount.toString()).to.equal(`${supply * 0.1}`);
      const fanoutAccountData = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
      expect(fanoutAccountData.totalShares?.toString()).to.equal(`${supply}`);
      expect(fanoutAccountData.totalStakedShares?.toString()).to.equal(`${supply * 0.1}`);
    });

    it('Allows Authority to Stake Members', async () => {
      const membershipMint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const supply = 1000000 * 10 ** 6;
      const member = new Keypair();
      await connection.requestAirdrop(member.publicKey, lamportsNeeded);
      const tokenAcct = await membershipMint.createAccount(authorityWallet.publicKey);
      await membershipMint.mintTo(tokenAcct, authorityWallet.publicKey, [], supply);

      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint.publicKey,
      });
      const ixs = await fanoutSdk.stakeForTokenMemberInstructions({
        shares: supply * 0.1,
        fanout: fanout,
        membershipMintTokenAccount: tokenAcct,
        membershipMint: membershipMint.publicKey,
        fanoutAuthority: authorityWallet.publicKey,
        member: member.publicKey,
        payer: authorityWallet.publicKey,
      });
      const tx = await fanoutSdk.sendInstructions(ixs.instructions, [], authorityWallet.publicKey);
      if (!!tx.RpcResponseAndContext.value.err) {
        const txdetails = await connection.getConfirmedTransaction(tx.TransactionSignature);
        console.log(txdetails, tx.RpcResponseAndContext.value.err);
      }
      const voucher = await fanoutSdk.fetch<FanoutMembershipVoucher>(
        ixs.output.membershipVoucher,
        FanoutMembershipVoucher,
      );

      expect(voucher.shares?.toString()).to.equal(`${supply * 0.1}`);
      expect(voucher.membershipKey?.toBase58()).to.equal(member.publicKey.toBase58());
      expect(voucher.fanout?.toBase58()).to.equal(fanout.toBase58());
      const stake = await membershipMint.getAccountInfo(ixs.output.stakeAccount);
      expect(stake.amount.toString()).to.equal(`${supply * 0.1}`);
      const fanoutAccountData = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
      expect(fanoutAccountData.totalShares?.toString()).to.equal(`${supply}`);
      expect(fanoutAccountData.totalStakedShares?.toString()).to.equal(`${supply * 0.1}`);
    });

    it('Distribute a Native Fanout with Token Members', async () => {
      const membershipMint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const distBot = new Keypair();
      await connection.requestAirdrop(distBot.publicKey, lamportsNeeded);
      const builtFanout = await builtTokenFanout(
        membershipMint,
        authorityWallet,
        fanoutSdk,
        100,
        5,
      );
      expect(builtFanout.fanoutAccountData.totalAvailableShares.toString()).to.equal('0');
      expect(builtFanout.fanoutAccountData.totalMembers.toString()).to.equal('5');
      expect(builtFanout.fanoutAccountData.totalShares?.toString()).to.equal(`${100 ** 6}`);
      expect(builtFanout.fanoutAccountData.totalStakedShares?.toString()).to.equal(`${100 ** 6}`);
      expect(builtFanout.fanoutAccountData.lastSnapshotAmount.toString()).to.equal('0');
      await connection.requestAirdrop(builtFanout.fanoutAccountData.accountKey, lamportsNeeded);
      const firstSnapshot = lamportsNeeded;
      const firstMemberAmount = firstSnapshot * 0.2;
      const member1 = builtFanout.members[0];
      const ix = await fanoutSdk.distributeTokenMemberInstructions({
        distributeForMint: false,
        membershipMint: membershipMint.publicKey,
        fanout: builtFanout.fanout,
        member: member1.wallet.publicKey,
        payer: distBot.publicKey,
      });
      const memberBefore = await fanoutSdk.connection.getAccountInfo(member1.wallet.publicKey);
      const tx = await fanoutSdk.sendInstructions(ix.instructions, [distBot], distBot.publicKey);

      if (!!tx.RpcResponseAndContext.value.err) {
        const txdetails = await connection.getConfirmedTransaction(tx.TransactionSignature);
        console.log(txdetails, tx.RpcResponseAndContext.value.err);
      }
      const voucher = await fanoutSdk.fetch<FanoutMembershipVoucher>(
        ix.output.membershipVoucher,
        FanoutMembershipVoucher,
      );
      const memberAfter = await fanoutSdk.connection.getAccountInfo(member1.wallet.publicKey);
      expect(voucher.lastInflow.toString()).to.equal(`${firstSnapshot}`);
      expect(voucher.shares.toString()).to.equal(`${100 ** 6 / 5}`);
      // @ts-ignore
      expect(memberAfter?.lamports - memberBefore?.lamports).to.equal(firstMemberAmount);
    });

    it('Unstake a Native Fanout with Token Members', async () => {
      const membershipMint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const distBot = new Keypair();
      const signature = await connection.requestAirdrop(distBot.publicKey, 1);
      await connection.confirmTransaction(signature);
      const builtFanout = await builtTokenFanout(
        membershipMint,
        authorityWallet,
        fanoutSdk,
        100,
        5,
      );
      const sent = 10;
      const beforeUnstake = await fanoutSdk.fetch<Fanout>(builtFanout.fanout, Fanout);
      await connection.requestAirdrop(builtFanout.fanoutAccountData.accountKey, sent);
      const firstSnapshot = sent * LAMPORTS_PER_SOL;
      //@ts-ignore
      const firstMemberAmount = firstSnapshot * 0.2;
      const member1 = builtFanout.members[0];

      const memberFanoutSdk = new FanoutClient(
        connection,
        new NodeWallet(new Account(member1.wallet.secretKey)),
      );
      const ix = await memberFanoutSdk.distributeTokenMemberInstructions({
        distributeForMint: false,
        membershipMint: membershipMint.publicKey,
        fanout: builtFanout.fanout,
        member: member1.wallet.publicKey,
        payer: member1.wallet.publicKey,
      });
      const voucherBefore = await memberFanoutSdk.fetch<FanoutMembershipVoucher>(
        ix.output.membershipVoucher,
        FanoutMembershipVoucher,
      );
      await memberFanoutSdk.unstakeTokenMember({
        fanout: builtFanout.fanout,
        member: member1.wallet.publicKey,
        payer: member1.wallet.publicKey,
      });
      const afterUnstake = await memberFanoutSdk.fetch<Fanout>(builtFanout.fanout, Fanout);
      //@ts-ignore
      const memberAfter = await memberFanoutSdk.connection.getAccountInfo(member1.wallet.publicKey);
      expect(afterUnstake.totalStakedShares?.toString()).to.equal(
        `${(beforeUnstake?.totalStakedShares as BN).sub(voucherBefore.shares as BN)}`,
      );
    });
  });
});
