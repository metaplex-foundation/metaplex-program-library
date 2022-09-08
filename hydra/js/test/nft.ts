/* eslint-disable @typescript-eslint/no-unused-vars */
import { Account, Connection, Keypair } from '@solana/web3.js';
import { NodeWallet } from '@project-serum/common'; //TODO remove this
import {
  NATIVE_MINT,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { expect, use } from 'chai';
import ChaiAsPromised from 'chai-as-promised';
import {
  Fanout,
  FanoutClient,
  FanoutMembershipMintVoucher,
  FanoutMembershipVoucher,
  FanoutMint,
  MembershipModel,
} from '../src';
import { createMasterEdition } from './utils/metaplex';
import { deprecated } from '@metaplex-foundation/mpl-token-metadata';
import { LOCALHOST } from '@metaplex-foundation/amman';
import { builtNftFanout } from './utils/scenarios';

use(ChaiAsPromised);

describe('fanout', async () => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const lamportsNeeded = 10000000000;
  let authorityWallet: Keypair;
  let fanoutSdk: FanoutClient;
  beforeEach(async () => {
    authorityWallet = Keypair.generate();
    let signature = await connection.requestAirdrop(authorityWallet.publicKey, 1000000000);
    await connection.confirmTransaction(signature);
    fanoutSdk = new FanoutClient(
      connection,
      new NodeWallet(new Account(authorityWallet.secretKey)),
    );
    signature = await connection.requestAirdrop(authorityWallet.publicKey, 1000000000);
    await connection.confirmTransaction(signature);
  });

  describe('NFT membership model', () => {
    describe('Creation', () => {
      it('Init', async () => {
        const { fanout } = await fanoutSdk.initializeFanout({
          totalShares: 100,
          name: `Test${Date.now()}`,
          membershipModel: MembershipModel.NFT,
        });

        const fanoutAccount = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
        expect(fanoutAccount.membershipModel).to.equal(MembershipModel.NFT);
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
          membershipModel: MembershipModel.NFT,
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
    });
    describe('Adding Members', () => {
      it('Adds Members With NFT', async () => {
        const init = await fanoutSdk.initializeFanout({
          totalShares: 100,
          name: `Test${Date.now()}`,
          membershipModel: MembershipModel.NFT,
        });
        const initMetadataData = new deprecated.DataV2({
          uri: 'URI',
          name: 'NAME',
          symbol: 'SYMBOL',
          sellerFeeBasisPoints: 1000,
          creators: null,
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
        const { membershipAccount } = await fanoutSdk.addMemberNft({
          fanout: init.fanout,
          fanoutNativeAccount: init.nativeAccount,
          membershipKey: nft.mint.publicKey,
          shares: 10,
        });
        const fanoutAccount = await fanoutSdk.fetch<Fanout>(init.fanout, Fanout);
        const membershipAccountData = await fanoutSdk.fetch<FanoutMembershipVoucher>(
          membershipAccount,
          FanoutMembershipVoucher,
        );
        expect(fanoutAccount.membershipModel).to.equal(MembershipModel.NFT);
        expect(fanoutAccount.lastSnapshotAmount.toString()).to.equal('0');
        expect(fanoutAccount.totalMembers.toString()).to.equal('1');
        expect(fanoutAccount.totalInflow.toString()).to.equal('0');
        expect(fanoutAccount.totalAvailableShares.toString()).to.equal('90');
        expect(fanoutAccount.totalShares.toString()).to.equal('100');
        expect(fanoutAccount.membershipMint).to.equal(null);
        expect(fanoutAccount.totalStakedShares).to.equal(null);
        expect(membershipAccountData?.shares?.toString()).to.equal('10');
        expect(membershipAccountData?.membershipKey?.toBase58()).to.equal(
          nft.mint.publicKey.toBase58(),
        );
      });

      it('Cannot Add mismatched Metadata', async () => {
        const init = await fanoutSdk.initializeFanout({
          totalShares: 100,
          name: `Test${Date.now()}`,
          membershipModel: MembershipModel.NFT,
        });
        const initMetadataData = new deprecated.DataV2({
          uri: 'URI',
          name: 'NAME',
          symbol: 'SYMBOL',
          sellerFeeBasisPoints: 1000,
          creators: null,
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

        const initMetadataData1 = new deprecated.DataV2({
          uri: 'URI1',
          name: 'NAME1',
          symbol: 'SYMBOL1',
          sellerFeeBasisPoints: 1000,
          creators: null,
          collection: null,
          uses: null,
        });
        //@ts-ignore
        const nft1 = await createMasterEdition(
          connection,
          authorityWallet,
          //@ts-ignore
          initMetadataData1,
          0,
        );
        const { membershipAccount } = await fanoutSdk.addMemberNft({
          fanout: init.fanout,
          fanoutNativeAccount: init.nativeAccount,
          membershipKey: nft.mint.publicKey,
          shares: 10,
        });
        const fanoutAccount = await fanoutSdk.fetch<Fanout>(init.fanout, Fanout);
        const membershipAccountData = await fanoutSdk.fetch<FanoutMembershipVoucher>(
          membershipAccount,
          FanoutMembershipVoucher,
        );
        expect(fanoutAccount.membershipModel).to.equal(MembershipModel.NFT);
        expect(fanoutAccount.lastSnapshotAmount.toString()).to.equal('0');
        expect(fanoutAccount.totalMembers.toString()).to.equal('1');
        expect(fanoutAccount.totalInflow.toString()).to.equal('0');
        expect(fanoutAccount.totalAvailableShares.toString()).to.equal('90');
        expect(fanoutAccount.totalShares.toString()).to.equal('100');
        expect(fanoutAccount.membershipMint).to.equal(null);
        expect(fanoutAccount.totalStakedShares).to.equal(null);
        expect(membershipAccountData?.shares?.toString()).to.equal('10');
        expect(membershipAccountData?.membershipKey?.toBase58()).to.equal(
          nft.mint.publicKey.toBase58(),
        );
      });
    });

    it('Distribute a Native Fanout with NFT Members', async () => {
      const builtFanout = await builtNftFanout(fanoutSdk, 100, 5);
      expect(builtFanout.fanoutAccountData.totalAvailableShares.toString()).to.equal('0');
      expect(builtFanout.fanoutAccountData.totalMembers.toString()).to.equal('5');
      expect(builtFanout.fanoutAccountData.lastSnapshotAmount.toString()).to.equal('0');
      const distBot = new Keypair();
      await connection.requestAirdrop(builtFanout.fanoutAccountData.accountKey, lamportsNeeded);
      await connection.requestAirdrop(distBot.publicKey, 1000000000);

      const member1 = builtFanout.members[0];
      const member2 = builtFanout.members[1];
      const distMember1 = await fanoutSdk.distributeNftMemberInstructions({
        distributeForMint: false,
        member: member1.wallet.publicKey,
        membershipKey: member1.mint,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
      });
      const distMember2 = await fanoutSdk.distributeNftMemberInstructions({
        distributeForMint: false,
        member: member2.wallet.publicKey,
        membershipKey: member2.mint,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
      });
      const memberDataBefore1 = await connection.getAccountInfo(member1.wallet.publicKey);
      const memberDataBefore2 = await connection.getAccountInfo(member2.wallet.publicKey);
      const holdingAccountBefore = await connection.getAccountInfo(
        builtFanout.fanoutAccountData.accountKey,
      );
      expect(memberDataBefore2).to.be.null;
      expect(memberDataBefore1).to.be.null;
      const holdingAccountReserved = await connection.getMinimumBalanceForRentExemption(1);
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
      const distAgainMember1 = await fanoutSdk.distributeNftMemberInstructions({
        distributeForMint: false,
        member: member1.wallet.publicKey,
        membershipKey: member1.mint,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
      });
      const distAgainMember1Tx = await fanoutSdk.sendInstructions(
        [...distAgainMember1.instructions],
        [distBot],
        distBot.publicKey,
      );
      await connection.getTransaction(distAgainMember1Tx.TransactionSignature);
      const memberDataAfterAgain1 = await connection.getAccountInfo(member1.wallet.publicKey);
      expect(memberDataAfterAgain1?.lamports).to.equal(firstSnapshot * 0.2);
      const membershipAccountAgain1 = await fanoutSdk.fetch<FanoutMembershipVoucher>(
        member1.voucher,
        FanoutMembershipVoucher,
      );
      expect(membershipAccountAgain1.totalInflow.toString()).to.equal(`${firstSnapshot * 0.2}`);
      const sent2 = lamportsNeeded;

      await connection.requestAirdrop(builtFanout.fanoutAccountData.accountKey, sent2);
      const secondInflow = sent2;
      await fanoutSdk.distributeAll({
        fanout: builtFanout.fanout,
        payer: fanoutSdk.wallet.publicKey,
        mint: NATIVE_MINT,
      });
      const memberDataAfterFinal1 = await connection.getAccountInfo(member1.wallet.publicKey);
      // @ts-ignore
      expect(memberDataAfterFinal1?.lamports).to.equal(
        memberDataAfter1?.lamports + secondInflow * 0.2,
      );
      const membershipAccountFinal1 = await fanoutSdk.fetch<FanoutMembershipVoucher>(
        member1.voucher,
        FanoutMembershipVoucher,
      );
      // @ts-ignore
      expect(membershipAccountFinal1?.totalInflow.toString()).to.equal(
        `${memberDataAfter1?.lamports + secondInflow * 0.2}`,
      );
    });

    it('Distributes a Fanout under a certain mint for NFT Members', async () => {
      const builtFanout = await builtNftFanout(fanoutSdk, 100, 5);

      const mint = await Token.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
        TOKEN_PROGRAM_ID,
      );
      const { fanoutForMint } = await fanoutSdk.initializeFanoutForMint({
        fanout: builtFanout.fanout,
        mint: mint.publicKey,
      });
      const fanoutForMintAccountData = await fanoutSdk.fetch<FanoutMint>(fanoutForMint, FanoutMint);
      const distBot = new Keypair();
      await connection.requestAirdrop(distBot.publicKey, lamportsNeeded);
      const sent = 112 * 1000000;
      await mint.mintTo(fanoutForMintAccountData.tokenAccount, authorityWallet, [], sent);
      const member1 = builtFanout.members[0];
      const member2 = builtFanout.members[1];
      const distMember1 = await fanoutSdk.distributeNftMemberInstructions({
        distributeForMint: true,
        member: member1.wallet.publicKey,
        membershipKey: member1.mint,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
        fanoutMint: mint.publicKey,
      });
      const distMember2 = await fanoutSdk.distributeNftMemberInstructions({
        distributeForMint: true,
        member: member2.wallet.publicKey,
        membershipKey: member2.mint,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
        fanoutMint: mint.publicKey,
      });
      const fanoutMintMember1TokenAccount = await Token.getAssociatedTokenAddress(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        mint.publicKey,
        member1.wallet.publicKey,
      );
      const fanoutMintMember2TokenAccount = await Token.getAssociatedTokenAddress(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        mint.publicKey,
        member2.wallet.publicKey,
      );
      const [fanoutForMintMembershipVoucher, _] = await FanoutClient.mintMembershipVoucher(
        fanoutForMint,
        member1.mint,
        mint.publicKey,
      );
      {
        const tx = await fanoutSdk.sendInstructions(
          [...distMember1.instructions, ...distMember2.instructions],
          [distBot],
          distBot.publicKey,
        );
        if (!!tx.RpcResponseAndContext.value.err) {
          const txdetails = await connection.getConfirmedTransaction(tx.TransactionSignature);
          console.log(txdetails, tx.RpcResponseAndContext.value.err);
        }
      }
      const fanoutForMintAccountDataAfter = await fanoutSdk.fetch<FanoutMint>(
        fanoutForMint,
        FanoutMint,
      );
      const fanoutForMintMember1VoucherAfter = await fanoutSdk.fetch<FanoutMembershipMintVoucher>(
        fanoutForMintMembershipVoucher,
        FanoutMembershipMintVoucher,
      );
      expect(
        (await connection.getTokenAccountBalance(fanoutMintMember1TokenAccount)).value.amount,
      ).to.equal(`${sent * 0.2}`);
      expect(
        (await connection.getTokenAccountBalance(fanoutMintMember2TokenAccount)).value.amount,
      ).to.equal(`${sent * 0.2}`);
      expect(fanoutForMintAccountDataAfter.totalInflow.toString()).to.equal(`${sent}`);
      expect(fanoutForMintAccountDataAfter.lastSnapshotAmount.toString()).to.equal(
        `${sent - sent * 0.2 * 2}`,
      );
      expect(fanoutForMintMember1VoucherAfter.lastInflow.toString()).to.equal(`${sent}`);
      const distMember1Again = await fanoutSdk.distributeNftMemberInstructions({
        distributeForMint: true,
        member: member1.wallet.publicKey,
        membershipKey: member1.mint,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
        fanoutMint: mint.publicKey,
      });
      await fanoutSdk.sendInstructions(
        [...distMember1Again.instructions],
        [distBot],
        distBot.publicKey,
      );
      await fanoutSdk.sendInstructions(
        [...distMember1Again.instructions],
        [distBot],
        distBot.publicKey,
      );
      expect(
        (await connection.getTokenAccountBalance(fanoutMintMember1TokenAccount)).value.amount,
      ).to.equal(`${sent * 0.2}`);
      const sent2 = 113 * 1000000;
      await mint.mintTo(fanoutForMintAccountData.tokenAccount, authorityWallet, [], sent2);
      const member3 = builtFanout.members[2];
      const distMember3 = await fanoutSdk.distributeNftMemberInstructions({
        distributeForMint: true,
        member: member3.wallet.publicKey,
        membershipKey: member3.mint,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
        fanoutMint: mint.publicKey,
      });

      const distMember1Final = await fanoutSdk.distributeNftMemberInstructions({
        distributeForMint: true,
        member: member1.wallet.publicKey,
        membershipKey: member1.mint,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
        fanoutMint: mint.publicKey,
      });
      const distMember2Final = await fanoutSdk.distributeNftMemberInstructions({
        distributeForMint: true,
        member: member2.wallet.publicKey,
        membershipKey: member2.mint,
        fanout: builtFanout.fanout,
        payer: distBot.publicKey,
        fanoutMint: mint.publicKey,
      });
      await fanoutSdk.sendInstructions(
        [
          ...distMember1Final.instructions,
          ...distMember2Final.instructions,
          ...distMember3.instructions,
        ],
        [distBot],
        distBot.publicKey,
      );
      const fanoutMintMember3TokenAccount = await Token.getAssociatedTokenAddress(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        mint.publicKey,
        member3.wallet.publicKey,
      );
      expect(
        (await connection.getTokenAccountBalance(fanoutMintMember1TokenAccount)).value.amount,
      ).to.equal(`${sent * 0.2 + sent2 * 0.2}`);
      expect(
        (await connection.getTokenAccountBalance(fanoutMintMember2TokenAccount)).value.amount,
      ).to.equal(`${sent * 0.2 + sent2 * 0.2}`);
      expect(
        (await connection.getTokenAccountBalance(fanoutMintMember3TokenAccount)).value.amount,
      ).to.equal(`${sent * 0.2 + sent2 * 0.2}`);
    });
  });
});
