import { Fanout, FanoutClient, MembershipModel } from '../../src';
import { createMasterEdition } from './metaplex';
import { deprecated } from '@metaplex-foundation/mpl-token-metadata';
import { Keypair, PublicKey, TransactionInstruction } from '@solana/web3.js';
import { Token } from '@solana/spl-token';
type BuiltNftFanout = {
  fanout: PublicKey;
  name: string;
  fanoutAccountData: Fanout;
  members: NftFanoutMember[];
};
type NftFanoutMember = {
  voucher: PublicKey;
  mint: PublicKey;
  wallet: Keypair;
};

type BuiltWalletFanout = {
  fanout: PublicKey;
  name: string;
  fanoutAccountData: Fanout;
  members: WalletFanoutMember[];
};
type WalletFanoutMember = {
  voucher: PublicKey;
  wallet: Keypair;
};

type StakedMembers = {
  voucher: PublicKey;
  stakeAccount: PublicKey;
  wallet: Keypair;
};

type BuiltTokenFanout = {
  fanout: PublicKey;
  name: string;
  membershipMint: Token;
  fanoutAccountData: Fanout;
  members: StakedMembers[];
};

export async function builtTokenFanout(
  mint: Token,
  mintAuth: Keypair,
  fanoutSdk: FanoutClient,
  shares: number,
  numberMembers: number,
): Promise<BuiltTokenFanout> {
  const name = `Test${Date.now()}`;
  const { fanout } = await fanoutSdk.initializeFanout({
    totalShares: 0,
    name: `Test${Date.now()}`,
    membershipModel: MembershipModel.Token,
    mint: mint.publicKey,
  });
  const mintInfo = await mint.getMintInfo();
  const totalSupply = shares ** mintInfo.decimals;
  const memberNumber = totalSupply / numberMembers;
  const members: StakedMembers[] = [];
  for (let i = 0; i < numberMembers; i++) {
    const memberWallet = new Keypair();
    await fanoutSdk.connection.requestAirdrop(memberWallet.publicKey, 10000000000);
    const ata = await mint.createAssociatedTokenAccount(memberWallet.publicKey);
    await mint.mintTo(ata, mintAuth, [], memberNumber);
    const ix = await fanoutSdk.stakeTokenMemberInstructions({
      shares: memberNumber,
      fanout: fanout,
      membershipMintTokenAccount: ata,
      membershipMint: mint.publicKey,
      member: memberWallet.publicKey,
      payer: memberWallet.publicKey,
    });
    console.log();
    const tx = await fanoutSdk.sendInstructions(
      ix.instructions,
      [memberWallet],
      memberWallet.publicKey,
    );
    if (!!tx.RpcResponseAndContext.value.err) {
      const txdetails = await fanoutSdk.connection.getConfirmedTransaction(tx.TransactionSignature);
      console.log(txdetails, tx.RpcResponseAndContext.value.err);
    }
    members.push({
      voucher: ix.output.membershipVoucher,
      stakeAccount: ix.output.stakeAccount,
      wallet: memberWallet,
    });
  }

  const fanoutAccount = await fanoutSdk.fetch<Fanout>(fanout, Fanout);
  return {
    fanout: fanout,
    name,
    membershipMint: mint,
    fanoutAccountData: fanoutAccount,
    members: members,
  };
}

export async function builtWalletFanout(
  fanoutSdk: FanoutClient,
  shares: number,
  numberMembers: number,
): Promise<BuiltWalletFanout> {
  const name = `Test${Date.now()}`;
  const init = await fanoutSdk.initializeFanout({
    totalShares: shares,
    name,
    membershipModel: MembershipModel.Wallet,
  });
  const memberNumber = shares / numberMembers;
  const ixs: TransactionInstruction[] = [];
  const members: WalletFanoutMember[] = [];
  for (let i = 0; i < numberMembers; i++) {
    const memberWallet = new Keypair();

    const ix = await fanoutSdk.addMemberWalletInstructions({
      fanout: init.fanout,
      fanoutNativeAccount: init.nativeAccount,
      membershipKey: memberWallet.publicKey,
      shares: memberNumber,
    });
    members.push({
      voucher: ix.output.membershipAccount,
      wallet: memberWallet,
    });
    ixs.push(...ix.instructions);
  }
  const tx = await fanoutSdk.sendInstructions(ixs, [], fanoutSdk.wallet.publicKey);
  if (!!tx.RpcResponseAndContext.value.err) {
    const txdetails = await fanoutSdk.connection.getConfirmedTransaction(tx.TransactionSignature);
    console.log(txdetails, tx.RpcResponseAndContext.value.err);
  }
  const fanoutAccount = await fanoutSdk.fetch<Fanout>(init.fanout, Fanout);
  return {
    fanout: init.fanout,
    name,
    fanoutAccountData: fanoutAccount,
    members: members,
  };
}

export async function builtNftFanout(
  fanoutSdk: FanoutClient,
  shares: number,
  numberMembers: number,
): Promise<BuiltNftFanout> {
  const name = `Test${Date.now()}`;
  const init = await fanoutSdk.initializeFanout({
    totalShares: shares,
    name,
    membershipModel: MembershipModel.NFT,
  });
  const memberNumber = shares / numberMembers;
  const ixs: TransactionInstruction[] = [];
  const members: NftFanoutMember[] = [];
  for (let i = 0; i < numberMembers; i++) {
    const memberWallet = new Keypair();
    const initMetadataData = new deprecated.DataV2({
      uri: 'URI' + i,
      name: 'NAME' + i,
      symbol: 'SYMBOL' + i,
      sellerFeeBasisPoints: 1000,
      creators: null,
      collection: null,
      uses: null,
    });
    const nft = await createMasterEdition(
      fanoutSdk.connection,
      //@ts-ignore
      fanoutSdk.wallet.payer,
      //@ts-ignore
      initMetadataData,
      0,
    );
    const tokenAccount = await nft.mint.getOrCreateAssociatedAccountInfo(memberWallet.publicKey);
    const owner = await nft.mint.getOrCreateAssociatedAccountInfo(fanoutSdk.wallet.publicKey);
    await nft.mint.transfer(
      owner.address,
      tokenAccount.address,
      //@ts-ignore
      fanoutSdk.wallet.payer,
      [],
      1,
    );
    const ix = await fanoutSdk.addMemberNftInstructions({
      fanout: init.fanout,
      fanoutNativeAccount: init.nativeAccount,
      membershipKey: nft.mint.publicKey,
      shares: memberNumber,
    });
    members.push({
      voucher: ix.output.membershipAccount,
      mint: nft.mint.publicKey,
      wallet: memberWallet,
    });
    ixs.push(...ix.instructions);
  }
  const tx = await fanoutSdk.sendInstructions(ixs, [], fanoutSdk.wallet.publicKey);
  if (!!tx.RpcResponseAndContext.value.err) {
    const txdetails = await fanoutSdk.connection.getConfirmedTransaction(tx.TransactionSignature);
    console.log(txdetails, tx.RpcResponseAndContext.value.err);
  }
  const fanoutAccount = await fanoutSdk.fetch<Fanout>(init.fanout, Fanout);
  return {
    fanout: init.fanout,
    name,
    fanoutAccountData: fanoutAccount,
    members: members,
  };
}
