/* eslint-disable @typescript-eslint/no-unused-vars */
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
} from '@solana/web3.js';
import * as splToken from '@solana/spl-token';
import { expect, use } from 'chai';
import ChaiAsPromised from 'chai-as-promised';
import { Fanout, FanoutClient, FanoutMembershipVoucher, FanoutMint, MembershipModel } from '../src';
import { LOCALHOST } from '@metaplex-foundation/amman';
import { builtTokenFanout } from './utils/scenarios';
import BN from 'bn.js';
import { Wallet } from '@project-serum/anchor';
import { ASSOCIATED_TOKEN_PROGRAM_ID, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import * as anchor from '@project-serum/anchor';
import uuid from 'short-uuid';
import {
  ThreadProgram as ThreadProgramType,
  IDL as ThreadProgramIdl_v1_3_15,
} from '../thread_program';
import * as hydraIdl from '../idl/hydra.json';
export const CLOCKWORK_THREAD_PROGRAM_ID = new PublicKey(
  '3XXuUFfweXBwFgFfYaejLvZE4cGZiHgKiGfMtdxNzYmv',
);
import fs from 'fs'
// function sleeps for ms ms 
async function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

use(ChaiAsPromised);

describe('fanout', async () => {
  const connection = new Connection(process.env.SOLANA_DEVNET, {commitment: 'confirmed', confirmTransactionInitialTimeout: 900000});
  const lamportsNeeded = 100000000;
  let authorityWallet: Keypair;
  let fanoutSdk: FanoutClient;
  beforeEach(async () => {
    authorityWallet = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync('/home/stacc/jaregm.json').toString())));
   // let signature = await (new Connection("https://api.devnet.solana.com")).requestAirdrop(authorityWallet.publicKey, lamportsNeeded);
   // //await connection.confirmTransaction(signature);
    fanoutSdk = new FanoutClient(connection, new Wallet(authorityWallet));
   // signature = await (new Connection("https://api.devnet.solana.com")).requestAirdrop(authorityWallet.publicKey, lamportsNeeded);
   // //await connection.confirmTransaction(signature);
  });

  describe('Token membership model', () => {
    it('Creates fanout w/ token, 2 members stake, has 5 random revenue events, and clockwork distributes', async () => {
      const membershipMint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const supply = 1000000 * 10 ** 6;
      const tokenAcct = await splToken.createAccount(
        connection,
        authorityWallet,
        membershipMint,
        authorityWallet.publicKey,
      );
      const { fanout, nativeAccount } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint,
      });
      const mint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const mintAcctAuthority = await splToken.createAssociatedTokenAccount(
        connection,
        authorityWallet,
        mint,
        authorityWallet.publicKey,
      );
      const { fanoutForMint, tokenAccount } = await fanoutSdk.initializeFanoutForMint({
        fanout,
        mint: mint,
      });

      const fanoutMintAccount = await fanoutSdk.fetch<FanoutMint>(fanoutForMint, FanoutMint);

      expect(fanoutMintAccount.mint.toBase58()).to.equal(mint.toBase58());
      expect(fanoutMintAccount.fanout.toBase58()).to.equal(fanout.toBase58());
      expect(fanoutMintAccount.tokenAccount.toBase58()).to.equal(tokenAccount.toBase58());
      expect(fanoutMintAccount.totalInflow.toString()).to.equal('0');
      expect(fanoutMintAccount.lastSnapshotAmount.toString()).to.equal('0');
      let totalStaked = 0;
      const members = [];
      let mtas = [];
      await splToken.mintTo(
        connection,
        authorityWallet,
        membershipMint,
        tokenAcct,
        authorityWallet,
        supply,
      );
      for (let index = 0; index <= 0; index++) {
        const member = new Keypair();
        const pseudoRng = Math.floor(supply * Math.random() * 0.138);
        await sleep(4000)
        await (new Connection("https://api.devnet.solana.com")).requestAirdrop(member.publicKey, lamportsNeeded);
        const tokenAcctMember = await splToken.createAssociatedTokenAccount(
          connection,
          authorityWallet,
          membershipMint,
          member.publicKey,
        );
        mtas.push(tokenAcctMember);
        const mintAcctMember = await splToken.createAssociatedTokenAccount(
          connection,
          authorityWallet,
          mint,
          member.publicKey,
        );
        await splToken.transfer(
          connection,
          authorityWallet,
          tokenAcct,
          tokenAcctMember,
          authorityWallet.publicKey,
          pseudoRng,
        );
        totalStaked += pseudoRng;
        const ixs = await fanoutSdk.stakeTokenMemberInstructions({
          shares: pseudoRng,
          fanout: fanout,
          membershipMintTokenAccount: tokenAcctMember,
          membershipMint: membershipMint,
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
        const stake = await splToken.getAccount(connection, ixs.output.stakeAccount);
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

      const SEED_QUEUE = 'thread';
      const distBot = Keypair.generate();
      let anotherTx = new Transaction();

      let signature2  = await (new Connection("https://api.devnet.solana.com")).requestAirdrop(distBot.publicKey, lamportsNeeded);
      await connection.confirmTransaction(signature2);
      await sleep(4000)
      const payerTokenAccount =
      (await splToken.getAssociatedTokenAddress(
        membershipMint,
        distBot.publicKey
      ));
      try {
        await connection.getTokenAccountBalance(payerTokenAccount);
      } catch (e) {
        anotherTx.add(
          await splToken.createAssociatedTokenAccountInstruction(
            
        distBot.publicKey,
            payerTokenAccount,
            distBot.publicKey,
            membershipMint,
          ),
        );
      }
      anotherTx.recentBlockhash = (await connection.getRecentBlockhash('max')).blockhash;
      anotherTx.feePayer = distBot.publicKey;
      let ahh = await connection.sendTransaction(anotherTx, [distBot]);
      await connection.confirmTransaction(ahh);

      await sleep(4000)
      let signature  = await (new Connection("https://api.devnet.solana.com")).requestAirdrop(distBot.publicKey, lamportsNeeded);
      await connection.confirmTransaction(signature);
      const threadProgram = await new anchor.Program(
        ThreadProgramIdl_v1_3_15,
        CLOCKWORK_THREAD_PROGRAM_ID,
        new anchor.AnchorProvider(connection, new Wallet(distBot), {}),
      );

      const hydraProgram = await new anchor.Program(
        hydraIdl as anchor.Idl,
        new PublicKey('13YzB1d3GKWNzKvk4hDzDzAmkLeNYXYWceQ46A652GVJ'),
        new anchor.AnchorProvider(connection, new Wallet(distBot), {}),
      );

      const { publicKey } = distBot;

      for (let index = 0; index <= 0; index++) {
        const threadName = uuid().new().substring(0, 8);
        const [pda] = await anchor.web3.PublicKey.findProgramAddress(
          [Buffer.from(SEED_QUEUE, 'utf-8'), publicKey.toBuffer(), Buffer.from(threadName, 'utf-8')],
          CLOCKWORK_THREAD_PROGRAM_ID,
        );
        let fakeTx = new Transaction();
        fakeTx.add(
          SystemProgram.transfer({
            fromPubkey: authorityWallet.publicKey,
            toPubkey: nativeAccount,
            lamports: 0.0138 * LAMPORTS_PER_SOL,
          }))
          fakeTx.feePayer = authorityWallet.publicKey;
          fakeTx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
          await connection.sendTransaction(fakeTx, [authorityWallet]);

          const member = members[index];

        try {
          const ix = await fanoutSdk.distributeTokenMemberInstructions({
            distributeForMint: false,
            membershipMint: membershipMint,
            fanout: fanout,
            member: member.member.publicKey,
            payer: distBot.publicKey
          });
          let transaction = new Transaction();
          for (var iiiii of ix.instructions){
            transaction.add(iiiii)
          }
          transaction.feePayer = distBot.publicKey;
          transaction.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
          let hmmm = await connection.sendTransaction(transaction, [distBot]);
          console.log(hmmm)
          const tokenAcctInfo = await connection.getTokenAccountBalance(
            member.fanoutMintTokenAccount,
            'confirmed',
          );
        } catch (err) {
          console.log(err);
        }
          fakeTx = new Transaction();
          fakeTx.add(
            SystemProgram.transfer({
              fromPubkey: authorityWallet.publicKey,
              toPubkey: nativeAccount,
              lamports: 0.0138 * LAMPORTS_PER_SOL,
            }))
            fakeTx.feePayer = authorityWallet.publicKey;
            fakeTx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
            await connection.sendTransaction(fakeTx, [authorityWallet]);
            fakeTx = new Transaction();
            fakeTx.add(
              SystemProgram.transfer({
                fromPubkey: authorityWallet.publicKey,
                toPubkey: nativeAccount,
                lamports: 0.0138 * LAMPORTS_PER_SOL,
              }))
              fakeTx.feePayer = authorityWallet.publicKey;
              fakeTx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
              await connection.sendTransaction(fakeTx, [authorityWallet]);
              fakeTx = new Transaction();
              fakeTx.add(
                SystemProgram.transfer({
                  fromPubkey: authorityWallet.publicKey,
                  toPubkey: nativeAccount,
                  lamports: 0.0138 * LAMPORTS_PER_SOL,
                }))
                fakeTx.feePayer = authorityWallet.publicKey;
                fakeTx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
                await connection.sendTransaction(fakeTx, [authorityWallet]);
            
        await sleep(4000)
      let signature  = await (new Connection("https://api.devnet.solana.com")).requestAirdrop(pda, lamportsNeeded);
      await connection.confirmTransaction(signature);
        console.log(pda.toBase58())
        const sent = Math.floor(Math.random() * 100 * 10 ** 6);
        await splToken.mintTo(
          connection,
          authorityWallet,
          mint,
          mintAcctAuthority,
          authorityWallet,
          sent,
        );
        await splToken.transfer(
          connection,
          authorityWallet,
          mintAcctAuthority,
          tokenAccount,
          authorityWallet,
          sent,
        );
        runningTotal += sent;

        try {
          const ix = await fanoutSdk.distributeClockTokenMemberInstructions({
            distributeForMint: false,
            hydra: pda,
            authority: authorityWallet.publicKey,
            membershipMint: membershipMint,
            fanout: fanout,
            member: member.member.publicKey,
            payer: distBot.publicKey
          });
          let transaction = new Transaction();
          for (var iiiii of ix.instructions){
            transaction.add(iiiii)
          }
          transaction.instructions = transaction.instructions.slice(0, transaction.instructions.length-1)
          transaction.feePayer = distBot.publicKey;
          transaction.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
          await connection.sendTransaction(transaction, [distBot]);
          await sleep(4000)
            console.log(ix.instructions[ix.instructions.length-1])
let winInst : any = ix.instructions[ix.instructions.length-1]
          const iix = await threadProgram.methods
            .threadCreate(
              threadName,
              {
                accounts: winInst.keys,
                programId: new PublicKey(winInst.programId),
                data: winInst.data,
              },
              {
                immediate: {}
              },
            )
            .accounts({
              authority: distBot.publicKey,
              payer: distBot.publicKey,
              thread: pda,
              systemProgram: anchor.web3.SystemProgram.programId,
            })
            .rpc();
          /*  const tx = await fanoutSdk.sendInstructions([iix], [distBot], distBot.publicKey);
            if (!!tx.RpcResponseAndContext.value.err) {
              const txdetails = await connection.getConfirmedTransaction(tx.TransactionSignature);
              console.log(txdetails, tx.RpcResponseAndContext.value.err);
            }
            console.log(tx) */
          const tokenAcctInfo = await connection.getTokenAccountBalance(
            member.fanoutMintTokenAccount,
            'confirmed',
          );
          const diff = ((supply - totalStaked) * sent) / totalStaked;
          const amountDist = (member.shares * diff) / supply;
        } catch (err) {
          console.log(err);
        }
      }
    });
    /*
  it('Creates fanout w/ token, 2 members stake, has 5 random revenue events, and distributes', async () => {
      const membershipMint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const distBot = new Keypair();
      await (new Connection("https://api.devnet.solana.com")).requestAirdrop(distBot.publicKey, lamportsNeeded);
      const supply = 1000000 * 10 ** 6;
      const tokenAcct = await splToken.createAccount(connection, authorityWallet, membershipMint, authorityWallet.publicKey);
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint,
      });
      const mint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const mintAcctAuthority = await splToken.createAssociatedTokenAccount(connection, authorityWallet, mint, authorityWallet.publicKey);
      const { fanoutForMint, tokenAccount } = await fanoutSdk.initializeFanoutForMint({
        fanout,
        mint: mint,
      });

      const fanoutMintAccount = await fanoutSdk.fetch<FanoutMint>(fanoutForMint, FanoutMint);

      expect(fanoutMintAccount.mint.toBase58()).to.equal(mint.toBase58());
      expect(fanoutMintAccount.fanout.toBase58()).to.equal(fanout.toBase58());
      expect(fanoutMintAccount.tokenAccount.toBase58()).to.equal(tokenAccount.toBase58());
      expect(fanoutMintAccount.totalInflow.toString()).to.equal('0');
      expect(fanoutMintAccount.lastSnapshotAmount.toString()).to.equal('0');
      let totalStaked = 0;
      const members = [];
      await splToken.mintTo(connection, authorityWallet, membershipMint, tokenAcct, authorityWallet, supply);
      for (let index = 0; index <= 0; index++) {
        const member = new Keypair();
        const pseudoRng = Math.floor(supply * Math.random() * 0.138);
        //await (new Connection("https://api.devnet.solana.com")).requestAirdrop(member.publicKey, lamportsNeeded);
        const tokenAcctMember = await splToken.createAssociatedTokenAccount(connection, authorityWallet,  membershipMint, member.publicKey);
        const mintAcctMember = await splToken.createAssociatedTokenAccount(connection, authorityWallet, mint, member.publicKey);
        await splToken.transfer(
          connection,
          authorityWallet,
          tokenAcct,
          tokenAcctMember,
          authorityWallet.publicKey,
          pseudoRng,
        );
        totalStaked += pseudoRng;
        const ixs = await fanoutSdk.stakeTokenMemberInstructions({
          shares: pseudoRng,
          fanout: fanout,
          membershipMintTokenAccount: tokenAcctMember,
          membershipMint: membershipMint,
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
        const stake = await splToken.getAccount(connection, ixs.output.stakeAccount);
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
      for (let index = 0; index <= 0; index++) {
        const sent = Math.floor(Math.random() * 100 * 10 ** 6);
        await splToken.mintTo(connection, authorityWallet, mint, mintAcctAuthority, authorityWallet, sent);
        await splToken.transfer(connection, authorityWallet, mintAcctAuthority, tokenAccount, authorityWallet, sent);
        runningTotal += sent;
        const member = members[index];
        const ix = await fanoutSdk.distributeTokenMemberInstructions({
          distributeForMint: true,
          fanoutMint: mint,
          membershipMint: membershipMint,
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
      const membershipMint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const supply = 1000000 * 10 ** 6;
      const tokenAcct = await splToken.createAccount(connection, authorityWallet, membershipMint, authorityWallet.publicKey);
      await splToken.mintTo(connection, authorityWallet, membershipMint, tokenAcct, authorityWallet, supply);
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint,
      });

      const fanoutAccount = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
      expect(fanoutAccount.membershipModel).to.equal(MembershipModel.Token);
      expect(fanoutAccount.lastSnapshotAmount.toString()).to.equal('0');
      expect(fanoutAccount.totalMembers.toString()).to.equal('0');
      expect(fanoutAccount.totalInflow.toString()).to.equal('0');
      expect(fanoutAccount.totalAvailableShares.toString()).to.equal('0');
      expect(fanoutAccount.totalShares.toString()).to.equal(supply.toString());
      expect(fanoutAccount.membershipMint?.toBase58()).to.equal(
        membershipMint.toBase58(),
      );
      expect(fanoutAccount.totalStakedShares?.toString()).to.equal('0');
    });

    it('Init For mint', async () => {
      const membershipMint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const supply = 1000000 * 10 ** 6;
      const tokenAcct = await splToken.createAccount(connection, authorityWallet, membershipMint, authorityWallet.publicKey);
      await splToken.mintTo(connection, authorityWallet, membershipMint, tokenAcct, authorityWallet, supply);
      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint,
      });
      const mint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const { fanoutForMint, tokenAccount } = await fanoutSdk.initializeFanoutForMint({
        fanout,
        mint: mint,
      });

      const fanoutMintAccount = await fanoutSdk.fetch<FanoutMint>(fanoutForMint, FanoutMint);

      expect(fanoutMintAccount.mint.toBase58()).to.equal(mint.toBase58());
      expect(fanoutMintAccount.fanout.toBase58()).to.equal(fanout.toBase58());
      expect(fanoutMintAccount.tokenAccount.toBase58()).to.equal(tokenAccount.toBase58());
      expect(fanoutMintAccount.totalInflow.toString()).to.equal('0');
      expect(fanoutMintAccount.lastSnapshotAmount.toString()).to.equal('0');
    });

    it('Stakes Members', async () => {
      const membershipMint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const supply = 1000000 * 10 ** 6;
      const member = new Keypair();
      //await (new Connection("https://api.devnet.solana.com")).requestAirdrop(member.publicKey, lamportsNeeded);
      const tokenAcct = await splToken.createAccount(connection, authorityWallet, membershipMint, authorityWallet.publicKey);
      const tokenAcctMember = await splToken.createAssociatedTokenAccount(connection, authorityWallet, membershipMint, member.publicKey);
      await splToken.mintTo(connection, authorityWallet, membershipMint, tokenAcct, authorityWallet, supply);
      await splToken.transfer(
        connection,
        authorityWallet,
        tokenAcct,
        tokenAcctMember,
        authorityWallet,
        supply * 0.1,
      );

      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint,
      });
      const ixs = await fanoutSdk.stakeTokenMemberInstructions({
        shares: supply * 0.1,
        fanout: fanout,
        membershipMintTokenAccount: tokenAcctMember,
        membershipMint: membershipMint,
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
      const stake = await splToken.getAccount(connection, ixs.output.stakeAccount);
      expect(stake.amount.toString()).to.equal(`${supply * 0.1}`);
      const fanoutAccountData = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
      expect(fanoutAccountData.totalShares?.toString()).to.equal(`${supply}`);
      expect(fanoutAccountData.totalStakedShares?.toString()).to.equal(`${supply * 0.1}`);
    });

    it('Allows Authority to Stake Members', async () => {
      const membershipMint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const supply = 1000000 * 10 ** 6;
      const member = new Keypair();
      //await (new Connection("https://api.devnet.solana.com")).requestAirdrop(member.publicKey, lamportsNeeded);
      const tokenAcct = await splToken.createAccount(connection, authorityWallet, membershipMint, authorityWallet.publicKey);
      await splToken.mintTo(connection, authorityWallet, membershipMint, tokenAcct, authorityWallet, supply);

      const { fanout } = await fanoutSdk.initializeFanout({
        totalShares: 0,
        name: `Test${Date.now()}`,
        membershipModel: MembershipModel.Token,
        mint: membershipMint,
      });
      const ixs = await fanoutSdk.stakeForTokenMemberInstructions({
        shares: supply * 0.1,
        fanout: fanout,
        membershipMintTokenAccount: tokenAcct,
        membershipMint: membershipMint,
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
      const stake = await splToken.getAccount(connection, ixs.output.stakeAccount);
      expect(stake.amount.toString()).to.equal(`${supply * 0.1}`);
      const fanoutAccountData = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
      expect(fanoutAccountData.totalShares?.toString()).to.equal(`${supply}`);
      expect(fanoutAccountData.totalStakedShares?.toString()).to.equal(`${supply * 0.1}`);
    });

    it('Distribute a Native Fanout with Token Members', async () => {
      const membershipMint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const distBot = new Keypair();
      await (new Connection("https://api.devnet.solana.com")).requestAirdrop(distBot.publicKey, lamportsNeeded);
      const builtFanout = await builtTokenFanout(
        new Token(connection,membershipMint, TOKEN_PROGRAM_ID, authorityWallet),
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
      await (new Connection("https://api.devnet.solana.com")).requestAirdrop(builtFanout.fanoutAccountData.accountKey, lamportsNeeded);
      const firstSnapshot = lamportsNeeded;
      const firstMemberAmount = firstSnapshot * 0.2;
      const member1 = builtFanout.members[0];
      const ix = await fanoutSdk.distributeTokenMemberInstructions({
        distributeForMint: false,
        membershipMint: membershipMint,
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
      const membershipMint = await splToken.createMint(
        connection,
        authorityWallet,
        authorityWallet.publicKey,
        null,
        6,
      );
      const distBot = new Keypair();
      const signature = await (new Connection("https://api.devnet.solana.com")).requestAirdrop(distBot.publicKey, 1);
      //await connection.confirmTransaction(signature);
      const builtFanout = await builtTokenFanout(
      new Token(connection,membershipMint, TOKEN_PROGRAM_ID, authorityWallet),
        authorityWallet,
        fanoutSdk,
        100,
        5,
      );
      const sent = 10;
      const beforeUnstake = await fanoutSdk.fetch<Fanout>(builtFanout.fanout, Fanout);
      await (new Connection("https://api.devnet.solana.com")).requestAirdrop(builtFanout.fanoutAccountData.accountKey, sent);
      const firstSnapshot = sent * LAMPORTS_PER_SOL;
      //@ts-ignore
      const firstMemberAmount = firstSnapshot * 0.2;
      const member1 = builtFanout.members[0];

      const memberFanoutSdk = new FanoutClient(connection, new Wallet(member1.wallet));
      const ix = await memberFanoutSdk.distributeTokenMemberInstructions({
        distributeForMint: false,
        membershipMint: membershipMint,
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
*/
  });
});
