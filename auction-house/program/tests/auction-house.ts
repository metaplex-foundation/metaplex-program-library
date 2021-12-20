import * as anchor from '@project-serum/anchor';
import { Wallet, createMint, createTokenAccount, getTokenAccount } from '@project-serum/common';
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
import { AuctionHouse } from '../../../target/types/auction_house';
import {
  AccountLayout,
  u64,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  Token,
  MintLayout,
} from '@solana/spl-token';
import { createMetadata } from '../../../token-metadata/js/test/actions';
import { PayerTransactionHandler } from '@metaplex-foundation/amman';
import { MetadataDataData } from '../../../token-metadata/js/src/mpl-token-metadata';

const AUCTION_HOUSE = 'auction_house';
const FEE_PAYER = 'fee_payer';
const TREASURY = 'treasury';
const SIGNER = 'signer';
const SFBP = 1000;

const WRAPPED_SOL_MINT = new PublicKey('So11111111111111111111111111111111111111112');

describe('mpl_auction_house', function () {
  // Configure the client to use the local cluster.
  const idl = JSON.parse(require('fs').readFileSync('./target/idl/auction_house.json', 'utf8'));

  const myWallet = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(require('fs').readFileSync(process.env.ANCHOR_WALLET, 'utf8'))),
  );

  const connection = new anchor.web3.Connection('http://127.0.0.1:8899/', 'confirmed');

  // Address of the deployed program.
  const programId = new anchor.web3.PublicKey('6obx1FPBafLhqxXL3AaqDcdv5f22j81x99QbTgbdsVA7');

  const walletWrapper: Wallet = new (anchor as any).Wallet(myWallet);

  const provider = new anchor.Provider(connection, walletWrapper, {
    preflightCommitment: 'recent',
  });

  // This handler is required to process actions from metaplex sdk
  // Actions - typescript wrappers under raw instructions
  const transactionHandler = new PayerTransactionHandler(provider.connection, myWallet);

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
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
  };

  const createAssociatedTokenAccount = async (
    transactionHandler: PayerTransactionHandler,
    mint: anchor.web3.PublicKey,
    associatedTokenAccount: anchor.web3.PublicKey,
    wallet: Wallet,
    payer: anchor.web3.Keypair,
  ) => {
    const tx = Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mint,
      associatedTokenAccount,
      wallet.publicKey,
      wallet.publicKey,
    );

    await transactionHandler.sendAndConfirmTransaction(new Transaction().add(tx), [payer]);
  };

  const mintTo = async (
    transactionHandler: PayerTransactionHandler,
    mint: anchor.web3.PublicKey,
    tokenAccount: anchor.web3.PublicKey,
    payer: anchor.web3.Keypair,
    amount: number,
  ) => {
    const tx = Token.createMintToInstruction(
      TOKEN_PROGRAM_ID,
      mint,
      tokenAccount,
      myWallet.publicKey,
      [],
      amount,
    );

    await transactionHandler.sendAndConfirmTransaction(new Transaction().add(tx), [payer]);
  };

  const getAuctionHouseFeeAcct = async (
    auctionHouse: anchor.web3.PublicKey,
  ): Promise<[PublicKey, number]> => {
    return await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(AUCTION_HOUSE), auctionHouse.toBuffer(), Buffer.from(FEE_PAYER)],
      programId,
    );
  };

  const getProgramAsSigner = async (): Promise<[anchor.web3.PublicKey, number]> => {
    return anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(AUCTION_HOUSE), Buffer.from(SIGNER)],
      programId,
    );
  };

  const getTradeState = async (
    wallet: anchor.web3.PublicKey,
    auctionHouse: anchor.web3.PublicKey,
    tokenAccount: anchor.web3.PublicKey,
    treasuryMint: anchor.web3.PublicKey,
    tokenAccountMint: anchor.web3.PublicKey,
    buyerPrice: anchor.BN,
    tokenSize: anchor.BN,
  ): Promise<[anchor.web3.PublicKey, number]> => {
    return anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(AUCTION_HOUSE),
        wallet.toBuffer(),
        auctionHouse.toBuffer(),
        tokenAccount.toBuffer(),
        treasuryMint.toBuffer(),
        tokenAccountMint.toBuffer(),
        buyerPrice.toBuffer('le', 8),
        tokenSize.toBuffer('le', 8),
      ],
      programId,
    );
  };

  const getEscrowPayment = async (
    auctionHouse: anchor.web3.PublicKey,
    wallet: anchor.web3.PublicKey,
  ): Promise<[anchor.web3.PublicKey, number]> => {
    return anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(AUCTION_HOUSE), auctionHouse.toBuffer(), wallet.toBuffer()],
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

  const transferLamports = async (
    transactionHandler: PayerTransactionHandler,
    to: anchor.web3.PublicKey,
    payer: anchor.web3.Keypair,
    amount: number,
  ) => {
    const tx = anchor.web3.SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: to,
      lamports: amount,
    });

    await transactionHandler.sendAndConfirmTransaction(new Transaction().add(tx), [payer]);
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

  it('Sell.', async function () {
    const buyerPrice: number = 1;
    const tokenSize: number = 1;

    // Obtain `AuctionHouse` native mint
    const treasuryMint = WRAPPED_SOL_MINT;

    // Create unique mint for `TokenMetadata`
    const tokenMint = await createMint(provider, myWallet.publicKey);

    // Create / Obtain associated token account for `AuctionHouse`
    const [tokenAccount] = await getAtaForMint(tokenMint, myWallet.publicKey);
    await createAssociatedTokenAccount(
      transactionHandler,
      tokenMint,
      tokenAccount,
      walletWrapper,
      myWallet,
    );

    mintTo(transactionHandler, tokenMint, tokenAccount, myWallet, 1);

    // Create `TokenMetadata`
    const { metadata } = await createMetadata({
      transactionHandler,
      publicKey: myWallet.publicKey,
      editionMint: tokenMint,
      metadataData: new MetadataDataData({
        creators: null,
        name: 'TOK - token',
        symbol: 'TOK',
        uri: 'https://github.com',
        sellerFeeBasisPoints: SFBP,
      }),
      updateAuthority: myWallet.publicKey,
    });

    // Obtain `AuctionHouse` state
    const [auctionHouse] = await getAuctionHouse(myWallet.publicKey, treasuryMint);

    // Obtain `AuctionHouse` fee account PDA
    const [auctionHouseFeeAccount] = await getAuctionHouseFeeAcct(auctionHouse);

    // Obtain `Program` authority with bump
    const [programAsSigner, programAsSignerBump] = await getProgramAsSigner();

    // Obtain seller trade state
    const [sellerTradeState, sellerTradeStateBump] = await getTradeState(
      myWallet.publicKey,
      auctionHouse,
      tokenAccount,
      treasuryMint,
      tokenMint,
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
    );

    // Obtain free seller trade state
    const [freeSellerTradeState, freeSellerTradeStateBump] = await getTradeState(
      myWallet.publicKey,
      auctionHouse,
      tokenAccount,
      treasuryMint,
      tokenMint,
      new anchor.BN(0),
      new anchor.BN(tokenSize),
    );

    // Transfer enough lamports to create seller trade state
    await transferLamports(transactionHandler, auctionHouseFeeAccount, myWallet, 10000000);

    // Call instruction by RPC
    await program.rpc.sell(
      new anchor.BN(sellerTradeStateBump),
      new anchor.BN(freeSellerTradeStateBump),
      new anchor.BN(programAsSignerBump),
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
      {
        accounts: {
          wallet: myWallet.publicKey,
          tokenAccount: tokenAccount,
          metadata: metadata,
          authority: myWallet.publicKey,
          auctionHouse: auctionHouse,
          auctionHouseFeeAccount: auctionHouseFeeAccount,
          sellerTradeState: sellerTradeState,
          freeSellerTradeState: freeSellerTradeState,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          programAsSigner: programAsSigner,
          rent: SYSVAR_RENT_PUBKEY,
        },
        signers: [],
      },
    );

    assert.equal(
      (await provider.connection.getAccountInfo(sellerTradeState)).data[0],
      sellerTradeStateBump,
    );
  });

  it('Cancel.', async function () {
    const buyerPrice: number = 1;
    const tokenSize: number = 1;

    // Obtain `AuctionHouse` native mint
    const treasuryMint = WRAPPED_SOL_MINT;

    // Create unique mint for `TokenMetadata`
    const tokenMint = await createMint(provider, myWallet.publicKey);

    // Create / Obtain associated token account for `AuctionHouse`
    const [tokenAccount] = await getAtaForMint(tokenMint, myWallet.publicKey);
    await createAssociatedTokenAccount(
      transactionHandler,
      tokenMint,
      tokenAccount,
      walletWrapper,
      myWallet,
    );

    mintTo(transactionHandler, tokenMint, tokenAccount, myWallet, 1);

    // Create `TokenMetadata`
    const { metadata } = await createMetadata({
      transactionHandler,
      publicKey: myWallet.publicKey,
      editionMint: tokenMint,
      metadataData: new MetadataDataData({
        creators: null,
        name: 'TOK - token',
        symbol: 'TOK',
        uri: 'https://github.com',
        sellerFeeBasisPoints: SFBP,
      }),
      updateAuthority: myWallet.publicKey,
    });

    // Obtain `AuctionHouse` state
    const [auctionHouse] = await getAuctionHouse(myWallet.publicKey, treasuryMint);

    // Obtain `AuctionHouse` fee account PDA
    const [auctionHouseFeeAccount] = await getAuctionHouseFeeAcct(auctionHouse);

    // Obtain `Program` authority with bump
    const [programAsSigner, programAsSignerBump] = await getProgramAsSigner();

    // Obtain seller trade state
    const [sellerTradeState, sellerTradeStateBump] = await getTradeState(
      myWallet.publicKey,
      auctionHouse,
      tokenAccount,
      treasuryMint,
      tokenMint,
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
    );

    // Obtain free seller trade state
    const [freeSellerTradeState, freeSellerTradeStateBump] = await getTradeState(
      myWallet.publicKey,
      auctionHouse,
      tokenAccount,
      treasuryMint,
      tokenMint,
      new anchor.BN(0),
      new anchor.BN(tokenSize),
    );

    // Transfer enough lamports to create seller trade state
    await transferLamports(transactionHandler, auctionHouseFeeAccount, myWallet, 10000000);

    // Call `Sell` instruction by RPC
    await program.rpc.sell(
      new anchor.BN(sellerTradeStateBump),
      new anchor.BN(freeSellerTradeStateBump),
      new anchor.BN(programAsSignerBump),
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
      {
        accounts: {
          wallet: myWallet.publicKey,
          tokenAccount: tokenAccount,
          metadata: metadata,
          authority: myWallet.publicKey,
          auctionHouse: auctionHouse,
          auctionHouseFeeAccount: auctionHouseFeeAccount,
          sellerTradeState: sellerTradeState,
          freeSellerTradeState: freeSellerTradeState,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          programAsSigner: programAsSigner,
          rent: SYSVAR_RENT_PUBKEY,
        },
        signers: [],
      },
    );

    const feePayerBeforeLamports = (
      await provider.connection.getAccountInfo(auctionHouseFeeAccount)
    ).lamports;
    const tradeStateLamports = (await provider.connection.getAccountInfo(sellerTradeState))
      .lamports;

    // Call `Cancel` instruction by RPC
    await program.rpc.cancel(new anchor.BN(buyerPrice), new anchor.BN(tokenSize), {
      accounts: {
        wallet: myWallet.publicKey,
        tokenAccount: tokenAccount,
        tokenMint: tokenMint,
        authority: myWallet.publicKey,
        auctionHouse: auctionHouse,
        auctionHouseFeeAccount: auctionHouseFeeAccount,
        tradeState: sellerTradeState,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [],
    });

    const feePayerLamports = (await provider.connection.getAccountInfo(auctionHouseFeeAccount))
      .lamports;

    assert.equal(feePayerBeforeLamports + tradeStateLamports, feePayerLamports);
    assert.equal(await provider.connection.getAccountInfo(sellerTradeState), null);
  });

  it('Buy', async function () {
    const buyerPrice: number = 1000;
    const tokenSize: number = 1;

    // Obtain `AuctionHouse` native mint
    const treasuryMint = WRAPPED_SOL_MINT;

    // Create unique mint for `TokenMetadata`
    const tokenMint = await createMint(provider, myWallet.publicKey);

    // Create / Obtain associated token account for `AuctionHouse`
    const [tokenAccount] = await getAtaForMint(tokenMint, myWallet.publicKey);
    await createAssociatedTokenAccount(
      transactionHandler,
      tokenMint,
      tokenAccount,
      walletWrapper,
      myWallet,
    );

    mintTo(transactionHandler, tokenMint, tokenAccount, myWallet, 1);

    // Create `TokenMetadata`
    const { metadata } = await createMetadata({
      transactionHandler,
      publicKey: myWallet.publicKey,
      editionMint: tokenMint,
      metadataData: new MetadataDataData({
        creators: null,
        name: 'TOK - token',
        symbol: 'TOK',
        uri: 'https://github.com',
        sellerFeeBasisPoints: SFBP,
      }),
      updateAuthority: myWallet.publicKey,
    });

    // Obtain `AuctionHouse` state PDA
    const [auctionHouse] = await getAuctionHouse(myWallet.publicKey, treasuryMint);

    // Obtain `AuctionHouse` fee account PDA
    const [auctionHouseFeeAccount] = await getAuctionHouseFeeAcct(auctionHouse);

    // Obtain escrow payment account PDA
    const [escrowPaymentAccount, escrowPaymentAccountBump] = await getEscrowPayment(
      auctionHouse,
      myWallet.publicKey,
    );

    // Obtain buyer trade state PDA
    const [buyerTradeState, buyerTradeStateBump] = await getTradeState(
      myWallet.publicKey,
      auctionHouse,
      tokenAccount,
      treasuryMint,
      tokenMint,
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
    );

    // Transfer enough lamports to create seller trade state
    await transferLamports(transactionHandler, auctionHouseFeeAccount, myWallet, 10000000);

    // Call instruction by RPC
    await program.rpc.buy(
      new anchor.BN(buyerTradeStateBump),
      new anchor.BN(escrowPaymentAccountBump),
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
      {
        accounts: {
          wallet: myWallet.publicKey,
          paymentAccount: myWallet.publicKey,
          transferAuthority: myWallet.publicKey,
          treasuryMint: treasuryMint,
          tokenAccount: tokenAccount,
          metadata: metadata,
          escrowPaymentAccount: escrowPaymentAccount,
          authority: myWallet.publicKey,
          auctionHouse: auctionHouse,
          auctionHouseFeeAccount: auctionHouseFeeAccount,
          buyerTradeState: buyerTradeState,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        },
        signers: [],
      },
    );

    assert.equal(
      (await provider.connection.getAccountInfo(buyerTradeState)).data[0],
      buyerTradeStateBump,
    );
  });

  it('Execute Sell.', async function () {
    const buyerPrice: number = 1;
    const tokenSize: number = 1;

    // Obtain `AuctionHouse` native mint
    const treasuryMint = WRAPPED_SOL_MINT;

    // Create unique mint for `TokenMetadata`
    const tokenMint = await createMint(provider, myWallet.publicKey);

    // Create buyer wallet
    const buyerWallet = new anchor.web3.Keypair();
    const buyerWalletWrapper: Wallet = new (anchor as any).Wallet(buyerWallet);

    // Transfer enough lamports to create buyer wallet
    await transferLamports(transactionHandler, buyerWallet.publicKey, myWallet, 1000000000);

    // Create / Obtain associated token account for `AuctionHouse`
    const [tokenAccount] = await getAtaForMint(tokenMint, myWallet.publicKey);
    await createAssociatedTokenAccount(
      transactionHandler,
      tokenMint,
      tokenAccount,
      walletWrapper,
      myWallet,
    );

    // Create / Obtain buyer associated token account for `AuctionHouse`
    const [buyerTokenAccount] = await getAtaForMint(tokenMint, buyerWallet.publicKey);
    await createAssociatedTokenAccount(
      transactionHandler,
      tokenMint,
      buyerTokenAccount,
      buyerWalletWrapper,
      buyerWallet,
    );

    mintTo(transactionHandler, tokenMint, tokenAccount, myWallet, 1);
    mintTo(transactionHandler, tokenMint, buyerTokenAccount, buyerWallet, 1);

    // Create `TokenMetadata`
    const { metadata } = await createMetadata({
      transactionHandler,
      publicKey: myWallet.publicKey,
      editionMint: tokenMint,
      metadataData: new MetadataDataData({
        creators: null,
        name: 'TOK - token',
        symbol: 'TOK',
        uri: 'https://github.com',
        sellerFeeBasisPoints: SFBP,
      }),
      updateAuthority: myWallet.publicKey,
    });

    // Obtain `AuctionHouse` state
    const [auctionHouse] = await getAuctionHouse(myWallet.publicKey, treasuryMint);

    // Obtain `AuctionHouse` fee account PDA
    const [auctionHouseFeeAccount] = await getAuctionHouseFeeAcct(auctionHouse);

    // Obtain `Program` authority with bump
    const [programAsSigner, programAsSignerBump] = await getProgramAsSigner();

    // Obtain seller trade state
    const [sellerTradeState, sellerTradeStateBump] = await getTradeState(
      myWallet.publicKey,
      auctionHouse,
      tokenAccount,
      treasuryMint,
      tokenMint,
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
    );

    // Obtain `AuctionHouse` treasury
    const [auctionHouseTreasury] = await getAuctionHouseTreasuryAcct(auctionHouse);

    // Obtain free seller trade state
    const [freeSellerTradeState, freeSellerTradeStateBump] = await getTradeState(
      myWallet.publicKey,
      auctionHouse,
      tokenAccount,
      treasuryMint,
      tokenMint,
      new anchor.BN(0),
      new anchor.BN(tokenSize),
    );

    // Transfer enough lamports to create seller trade state
    await transferLamports(transactionHandler, auctionHouseFeeAccount, myWallet, 10000000);

    // Call `Sell` instruction by RPC
    await program.rpc.sell(
      new anchor.BN(sellerTradeStateBump),
      new anchor.BN(freeSellerTradeStateBump),
      new anchor.BN(programAsSignerBump),
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
      {
        accounts: {
          wallet: myWallet.publicKey,
          tokenAccount: tokenAccount,
          metadata: metadata,
          authority: myWallet.publicKey,
          auctionHouse: auctionHouse,
          auctionHouseFeeAccount: auctionHouseFeeAccount,
          sellerTradeState: sellerTradeState,
          freeSellerTradeState: freeSellerTradeState,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          programAsSigner: programAsSigner,
          rent: SYSVAR_RENT_PUBKEY,
        },
        signers: [],
      },
    );

    // Obtain escrow payment account PDA
    const [escrowPaymentAccount, escrowPaymentAccountBump] = await getEscrowPayment(
      auctionHouse,
      buyerWallet.publicKey,
    );

    // Transfer enough lamports to create escrow account
    // This account will be created on-chain if provided mint is not native(WRAPPED_SOL_MINT)
    await transferLamports(transactionHandler, escrowPaymentAccount, myWallet, 1000000000);

    // Obtain buyer trade state PDA
    const [buyerTradeState, buyerTradeStateBump] = await getTradeState(
      buyerWallet.publicKey,
      auctionHouse,
      tokenAccount,
      treasuryMint,
      tokenMint,
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
    );

    // Call `Buy` instruction by RPC
    await program.rpc.buy(
      new anchor.BN(buyerTradeStateBump),
      new anchor.BN(escrowPaymentAccountBump),
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
      {
        accounts: {
          wallet: buyerWallet.publicKey,
          paymentAccount: buyerWallet.publicKey,
          transferAuthority: buyerWallet.publicKey,
          treasuryMint: treasuryMint,
          tokenAccount: tokenAccount,
          metadata: metadata,
          escrowPaymentAccount: escrowPaymentAccount,
          authority: myWallet.publicKey,
          auctionHouse: auctionHouse,
          auctionHouseFeeAccount: auctionHouseFeeAccount,
          buyerTradeState: buyerTradeState,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        },
        signers: [buyerWallet],
      },
    );

    const escrowLamportsBefore = (await provider.connection.getAccountInfo(escrowPaymentAccount))
      .lamports;
    const buyerTokenAccountBefore = (await getTokenAccount(provider, buyerTokenAccount)).amount;

    // Call `ExecuteSell` instruction by RPC
    await program.rpc.executeSale(
      new anchor.BN(escrowPaymentAccountBump),
      new anchor.BN(freeSellerTradeStateBump),
      new anchor.BN(programAsSignerBump),
      new anchor.BN(buyerPrice),
      new anchor.BN(tokenSize),
      {
        accounts: {
          buyer: buyerWallet.publicKey,
          seller: myWallet.publicKey,
          tokenAccount: tokenAccount,
          tokenMint: tokenMint,
          metadata: metadata,
          treasuryMint: treasuryMint,
          escrowPaymentAccount: escrowPaymentAccount,
          sellerPaymentReceiptAccount: myWallet.publicKey,
          buyerReceiptTokenAccount: buyerTokenAccount,
          authority: myWallet.publicKey,
          auctionHouse: auctionHouse,
          auctionHouseFeeAccount: auctionHouseFeeAccount,
          auctionHouseTreasury: auctionHouseTreasury,
          buyerTradeState: buyerTradeState,
          sellerTradeState: sellerTradeState,
          freeTradeState: freeSellerTradeState,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          ataProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          programAsSigner: programAsSigner,
          rent: SYSVAR_RENT_PUBKEY,
        },
        signers: [myWallet],
      },
    );

    const escrowLamportsAfter = (await provider.connection.getAccountInfo(escrowPaymentAccount))
      .lamports;
    const buyerTokenAccountAfter = (await getTokenAccount(provider, buyerTokenAccount)).amount;

    assert.equal(escrowLamportsAfter, escrowLamportsBefore - tokenSize);
    assert.equal(
      buyerTokenAccountAfter.toNumber(),
      buyerTokenAccountBefore.add(new anchor.BN(tokenSize)).toNumber(),
    );
  });

  it("Deposit.", async function(){
    const amount = 5;
    
    // Obtain `AuctionHouse` native mint
    const tMintKey = WRAPPED_SOL_MINT;
    
    const [auctionHouseKey, bump] = await getAuctionHouse(myWallet.publicKey, tMintKey);
    const auctionHouseObj = await program.account.auctionHouse.fetch(auctionHouseKey);
    
    const amountAdjusted = await getPriceWithMantissa(
      amount,
      auctionHouseObj.treasuryMint,
      myWallet,
      program,
    );
  
    const [escrowPaymentAccount, escrowPaymentAccountBump] = await getAuctionHouseBuyerEscrow(
      auctionHouseKey,
      myWallet.publicKey,
    );
  
    // Make two deposit operation then subtract the lamport balance 
    // after second deposit from the one after first one.
    // The substraction result should be equal second deposit amount.
    await program.rpc.deposit(escrowPaymentAccountBump, new anchor.BN(amountAdjusted), {
      accounts: {
        wallet: myWallet.publicKey,
        paymentAccount: myWallet.publicKey,
        transferAuthority: anchor.web3.SystemProgram.programId,
        escrowPaymentAccount,
        treasuryMint: auctionHouseObj.treasuryMint,
        authority: auctionHouseObj.authority,
        auctionHouse: auctionHouseKey,
        auctionHouseFeeAccount: auctionHouseObj.auctionHouseFeeAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      }
    });

    const escrowPaymentAccountAfterFirstDeposit = 
      (await provider.connection.getAccountInfo(escrowPaymentAccount)).lamports;

    await program.rpc.deposit(escrowPaymentAccountBump, new anchor.BN(amountAdjusted), {
      accounts: {
        wallet: myWallet.publicKey,
        paymentAccount: myWallet.publicKey,
        transferAuthority: anchor.web3.SystemProgram.programId,
        escrowPaymentAccount,
        treasuryMint: auctionHouseObj.treasuryMint,
        authority: auctionHouseObj.authority,
        auctionHouse: auctionHouseKey,
        auctionHouseFeeAccount: auctionHouseObj.auctionHouseFeeAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      }
    });

    const escrowPaymentAccountAfterSecondDeposit = 
      (await provider.connection.getAccountInfo(escrowPaymentAccount)).lamports;

    assert.equal(
      escrowPaymentAccountAfterSecondDeposit - escrowPaymentAccountAfterFirstDeposit, 
      new anchor.BN(amountAdjusted).toNumber()
    );

  });

  it("Withdraw.", async function () {
    const amount = 5;
    
    // Obtain `AuctionHouse` native mint
    const tMintKey = WRAPPED_SOL_MINT;
    
    const [auctionHouseKey, bump] = await getAuctionHouse(myWallet.publicKey, tMintKey);
    const auctionHouseObj = await program.account.auctionHouse.fetch(auctionHouseKey);
    
    const amountAdjusted = await getPriceWithMantissa(
      amount,
      auctionHouseObj.treasuryMint,
      myWallet,
      program,
    );
  
    const [escrowPaymentAccount, escrowPaymentAccountBump] = await getAuctionHouseBuyerEscrow(
      auctionHouseKey,
      myWallet.publicKey,
    );

    const escrowPaymentAccountBeforeWithdraw = 
      (await provider.connection.getAccountInfo(escrowPaymentAccount)).lamports;

    await program.rpc.withdraw(escrowPaymentAccountBump, new anchor.BN(amountAdjusted), {
      accounts: {
        wallet: myWallet.publicKey,
        receiptAccount: myWallet.publicKey,
        escrowPaymentAccount,
        treasuryMint: auctionHouseObj.treasuryMint,
        authority: auctionHouseObj.authority,
        auctionHouse: auctionHouseKey,
        auctionHouseFeeAccount: auctionHouseObj.auctionHouseFeeAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        ataProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      }
    });

    const escrowPaymentAccountAfterWithdraw = 
      (await provider.connection.getAccountInfo(escrowPaymentAccount)).lamports;

    assert.equal(escrowPaymentAccountBeforeWithdraw - escrowPaymentAccountAfterWithdraw, amountAdjusted);

  });

});
