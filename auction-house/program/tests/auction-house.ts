import * as anchor from '@project-serum/anchor';
import { Wallet } from '@project-serum/common';
import assert from 'assert';
import {
  Blockhash,
  PublicKey,
  FeeCalculator,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
} from '@solana/web3.js';
import { ASSOCIATED_TOKEN_PROGRAM_ID, Token } from '@solana/spl-token';
import { AuctionHouse } from '../../../target/types/auction_house';
import { AccountLayout, u64 } from '@solana/spl-token';

const AUCTION_HOUSE = 'auction_house';
const FEE_PAYER = 'fee_payer';
const TREASURY = 'treasury';
const SFBP = 1000;

const WRAPPED_SOL_MINT = new PublicKey('So11111111111111111111111111111111111111112');
const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);

describe('metaplex_auction_house', function () {
  // Configure the client to use the local cluster.
  const idl = JSON.parse(
    require('fs').readFileSync('../../../target/idl/auction_house.json', 'utf8'),
  );

  const myWallet = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(require('fs').readFileSync(process.env.ANCHOR_WALLET, 'utf8'))),
  );

  const connection = new anchor.web3.Connection('http://127.0.0.1:8899/', 'recent');

  // Address of the deployed program.
  const programId = new anchor.web3.PublicKey('6Dtd5LSEZo3evRzWCjydocMJXtthFJKjcr8AnHdYrEuB');

  const walletWrapper: Wallet = new (anchor as any).Wallet(myWallet);

  const provider = new anchor.Provider(connection, walletWrapper, {
    preflightCommitment: 'recent',
  });

  const program = new anchor.Program(idl, programId, provider);

  const getAuctionHouse = async (
    creator: anchor.web3.PublicKey,
    treasuryMint: anchor.web3.PublicKey,
  ): Promise<[PublicKey, number]> => {
    return await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(AUCTION_HOUSE), creator.toBuffer(), treasuryMint.toBuffer()],
      programId,
    );
  };

  const getAtaForMint = async (
    mint: anchor.web3.PublicKey,
    buyer: anchor.web3.PublicKey,
  ): Promise<[anchor.web3.PublicKey, number]> => {
    return await anchor.web3.PublicKey.findProgramAddress(
      [buyer.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    );
  };

  const getAuctionHouseFeeAcct = async (
    auctionHouse: anchor.web3.PublicKey,
  ): Promise<[PublicKey, number]> => {
    return await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(AUCTION_HOUSE), auctionHouse.toBuffer(), Buffer.from(FEE_PAYER)],
      programId,
    );
  };

  const getAuctionHouseTreasuryAcct = async (
    auctionHouse: anchor.web3.PublicKey,
  ): Promise<[PublicKey, number]> => {
    return await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(AUCTION_HOUSE), auctionHouse.toBuffer(), Buffer.from(TREASURY)],
      programId,
    );
  };

  const deserializeAccount = (data: Buffer) => {
    const accountInfo = AccountLayout.decode(data);
    accountInfo.mint = new PublicKey(accountInfo.mint);
    accountInfo.owner = new PublicKey(accountInfo.owner);
    accountInfo.amount = u64.fromBuffer(accountInfo.amount);

    if (accountInfo.delegateOption === 0) {
      accountInfo.delegate = null;
      accountInfo.delegatedAmount = new u64(0);
    } else {
      accountInfo.delegate = new PublicKey(accountInfo.delegate);
      accountInfo.delegatedAmount = u64.fromBuffer(accountInfo.delegatedAmount);
    }

    accountInfo.isInitialized = accountInfo.state !== 0;
    accountInfo.isFrozen = accountInfo.state === 2;

    if (accountInfo.isNativeOption === 1) {
      accountInfo.rentExemptReserve = u64.fromBuffer(accountInfo.isNative);
      accountInfo.isNative = true;
    } else {
      accountInfo.rentExemptReserve = null;
      accountInfo.isNative = false;
    }

    if (accountInfo.closeAuthorityOption === 0) {
      accountInfo.closeAuthority = null;
    } else {
      accountInfo.closeAuthority = new PublicKey(accountInfo.closeAuthority);
    }

    return accountInfo;
  };

  const getPriceWithMantissa = async (
    price: number,
    mint: anchor.web3.PublicKey,
    walletKeyPair: any,
    anchorProgram: anchor.Program,
  ): Promise<number> => {
    const token = new Token(
      anchorProgram.provider.connection,
      new anchor.web3.PublicKey(mint),
      TOKEN_PROGRAM_ID,
      walletKeyPair,
    );

    const mintInfo = await token.getMintInfo();

    const mantissa = 10 ** mintInfo.decimals;

    return Math.ceil(price * mantissa);
  };

  const getAuctionHouseBuyerEscrow = async (
    auctionHouse: anchor.web3.PublicKey,
    wallet: anchor.web3.PublicKey,
  ): Promise<[PublicKey, number]> => {
    return await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(AUCTION_HOUSE), auctionHouse.toBuffer(), wallet.toBuffer()],
      programId,
    );
  };

  async function getTokenAmount(
    anchorProgram: anchor.Program,
    account: anchor.web3.PublicKey,
    mint: anchor.web3.PublicKey,
  ): Promise<number> {
    let amount = 0;
    if (!mint.equals(WRAPPED_SOL_MINT)) {
      try {
        const token = await anchorProgram.provider.connection.getTokenAccountBalance(account);
        amount = token.value.uiAmount * Math.pow(10, token.value.decimals);
      } catch (e) {
        console.log(e);
        console.log('Account ', account.toBase58(), 'didnt return value. Assuming 0 tokens.');
      }
    } else {
      amount = await anchorProgram.provider.connection.getBalance(account);
    }
    return amount;
  }

  /////////////////////////////////////////////////////////////////////////////////////
  beforeEach(async function () {
    let twdKey: anchor.web3.PublicKey;
    let fwdKey: anchor.web3.PublicKey;
    let tMintKey: anchor.web3.PublicKey;

    twdKey = myWallet.publicKey;
    fwdKey = myWallet.publicKey;
    tMintKey = WRAPPED_SOL_MINT;

    const twdAta = tMintKey.equals(WRAPPED_SOL_MINT)
      ? twdKey
      : (await getAtaForMint(tMintKey, twdKey))[0];

    const [auctionHouse, bump] = await getAuctionHouse(myWallet.publicKey, tMintKey);

    const [feeAccount, feeBump] = await getAuctionHouseFeeAcct(auctionHouse);
    const [treasuryAccount, treasuryBump] = await getAuctionHouseTreasuryAcct(auctionHouse);

    try {
      const house = await program.account.auctionHouse.fetch(auctionHouse);
    } catch (e) {
      await program.rpc.createAuctionHouse(bump, feeBump, treasuryBump, SFBP, 'true', 'true', {
        accounts: {
          treasuryMint: tMintKey,
          payer: myWallet.publicKey,
          authority: myWallet.publicKey,
          feeWithdrawalDestination: fwdKey,
          treasuryWithdrawalDestination: twdAta,
          treasuryWithdrawalDestinationOwner: twdKey,
          auctionHouse,
          auctionHouseFeeAccount: feeAccount,
          auctionHouseTreasury: treasuryAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          ataProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
      });
    }
  });

  it('Create Auction House.', async function () {
    const tMintKey = WRAPPED_SOL_MINT;
    const [auctionHouse, bump] = await getAuctionHouse(myWallet.publicKey, tMintKey);

    const [feeAccount, feeBump] = await getAuctionHouseFeeAcct(auctionHouse);
    const [treasuryAccount, treasuryBump] = await getAuctionHouseTreasuryAcct(auctionHouse);

    const house = await program.account.auctionHouse.fetch(auctionHouse);

    assert.ok(house.auctionHouseFeeAccount.equals(feeAccount));
    assert.ok(house.auctionHouseTreasury.equals(treasuryAccount));
    assert.ok(house.treasuryWithdrawalDestination.equals(myWallet.publicKey));
    assert.ok(house.feeWithdrawalDestination.equals(myWallet.publicKey));
    assert.ok(house.treasuryMint.equals(tMintKey));
    assert.ok(house.authority.equals(myWallet.publicKey));
    assert.ok(house.creator.equals(myWallet.publicKey));
    assert.equal(house.bump, bump);
    assert.equal(house.feePayerBump, feeBump);
    assert.equal(house.treasuryBump, treasuryBump);
    assert.equal(house.sellerFeeBasisPoints, SFBP);
    assert.equal(house.requiresSignOff, true);
    assert.equal(house.canChangeSalePrice, true);
  });

  it('Update Auction House.', async function () {
    let tMintKey: anchor.web3.PublicKey;
    tMintKey = WRAPPED_SOL_MINT;

    const [auctionHouseKey, bump] = await getAuctionHouse(myWallet.publicKey, tMintKey);

    const auctionHouseObj = await program.account.auctionHouse.fetch(auctionHouseKey);
    tMintKey = auctionHouseObj.treasuryMint;
    let twdKey: anchor.web3.PublicKey;
    let fwdKey: anchor.web3.PublicKey;

    twdKey = tMintKey.equals(WRAPPED_SOL_MINT)
      ? auctionHouseObj.treasuryWithdrawalDestination
      : deserializeAccount(
          Buffer.from(
            (
              await program.provider.connection.getAccountInfo(
                auctionHouseObj.treasuryWithdrawalDestination,
              )
            ).data,
          ),
        ).owner;

    fwdKey = auctionHouseObj.feeWithdrawalDestination;

    const twdAta = tMintKey.equals(WRAPPED_SOL_MINT)
      ? twdKey
      : (await getAtaForMint(tMintKey, twdKey))[0];

    let sfbp = auctionHouseObj.sellerFeeBasisPoints;
    let newAuth = auctionHouseObj.authority;
    let ccsp = auctionHouseObj.canChangeSalePrice;
    let rso = auctionHouseObj.requiresSignOff;

    await program.rpc.updateAuctionHouse(SFBP * 2, false, false, {
      accounts: {
        treasuryMint: tMintKey,
        payer: myWallet.publicKey,
        authority: myWallet.publicKey,
        newAuthority: newAuth,
        feeWithdrawalDestination: fwdKey,
        treasuryWithdrawalDestination: twdAta,
        treasuryWithdrawalDestinationOwner: twdKey,
        auctionHouse: auctionHouseKey,
        auctionHouseFeeAccount: auctionHouseObj.auctionHouseFeeAccount,
        auctionHouseTreasury: auctionHouseObj.auctionHouseTreasury,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        ataProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
    });

    const house = await program.account.auctionHouse.fetch(auctionHouseKey);

    assert.equal(house.sellerFeeBasisPoints, SFBP * 2);
    assert.equal(house.requiresSignOff, false);
    assert.equal(house.canChangeSalePrice, false);

    await program.rpc.updateAuctionHouse(SFBP, true, true, {
      accounts: {
        treasuryMint: tMintKey,
        payer: myWallet.publicKey,
        authority: myWallet.publicKey,
        newAuthority: newAuth,
        feeWithdrawalDestination: fwdKey,
        treasuryWithdrawalDestination: twdAta,
        treasuryWithdrawalDestinationOwner: twdKey,
        auctionHouse: auctionHouseKey,
        auctionHouseFeeAccount: auctionHouseObj.auctionHouseFeeAccount,
        auctionHouseTreasury: auctionHouseObj.auctionHouseTreasury,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        ataProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
    });
  });
});
