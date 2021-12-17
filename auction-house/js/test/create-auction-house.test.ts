import test from 'tape';
import * as anchor from "@project-serum/anchor";
import { 
  killStuckProcess, 
  getAtaForMint,
  getAuctionHouse,
  getAuctionHouseFeeAcct, 
  getAuctionHouseTreasuryAcct,
  programId,
  WRAPPED_SOL_MINT, 
  TOKEN_PROGRAM_ID,
  SFBP
} from './utils';
import { ASSOCIATED_TOKEN_PROGRAM_ID, Token } from '@solana/spl-token';

killStuckProcess();

test('create-auction-house: success', async (t) => {
  const idl = JSON.parse(require('fs').readFileSync('./idl/auction_house.json', 'utf8'));
  const myWallet = anchor.web3.Keypair.fromSecretKey(new Uint8Array(
    JSON.parse(require("fs").readFileSync(process.env.ANCHOR_WALLET, "utf8"))
  ));
  const program = new anchor.Program(idl, programId);

  let twdKey: anchor.web3.PublicKey;
  let fwdKey: anchor.web3.PublicKey;
  let tMintKey: anchor.web3.PublicKey;
  
  twdKey = myWallet.publicKey;
  fwdKey = myWallet.publicKey;
  tMintKey = WRAPPED_SOL_MINT;

  const twdAta = tMintKey.equals(WRAPPED_SOL_MINT)
  ? twdKey
   : (await getAtaForMint(tMintKey, twdKey))[0];

  const [auctionHouse, bump] = await getAuctionHouse(
    myWallet.publicKey,
    tMintKey,
  );

  const [feeAccount, feeBump] = await getAuctionHouseFeeAcct(auctionHouse);
  const [treasuryAccount, treasuryBump] = await getAuctionHouseTreasuryAcct(auctionHouse);

  try {
    const house = await program.account.auctionHouse.fetch(auctionHouse);
  } catch (e) {
    await program.rpc.createAuctionHouse(
      bump,
      feeBump,
      treasuryBump,
      SFBP,
      'true',
      'true',
      {
        accounts: {
          treasuryMint: tMintKey,
          payer: myWallet.publicKey,
          authority: myWallet.publicKey,
          feeWithdrawalDestination: fwdKey,
          treasuryWithdrawalDestination: twdAta,
          treasuryWithdrawalDestinationOwner: twdKey,
          auctionHouse: auctionHouse,
          auctionHouseFeeAccount: feeAccount,
          auctionHouseTreasury: treasuryAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          ataProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
      },
    );
  }
  
  const house = await program.account.auctionHouse.fetch(auctionHouse);

  t.ok(house.auctionHouseFeeAccount.equals(feeAccount));
  t.ok(house.auctionHouseTreasury.equals(treasuryAccount));
  t.ok(house.treasuryWithdrawalDestination.equals(myWallet.publicKey));
  t.ok(house.feeWithdrawalDestination.equals(myWallet.publicKey));
  t.ok(house.treasuryMint.equals(tMintKey));
  t.ok(house.authority.equals(myWallet.publicKey));
  t.ok(house.creator.equals(myWallet.publicKey));
  t.equal(house.bump, bump);
  t.equal(house.feePayerBump, feeBump);
  t.equal(house.treasuryBump, treasuryBump);
  t.equal(house.sellerFeeBasisPoints, SFBP);
  t.equal(house.requiresSignOff, true);
  t.equal(house.canChangeSalePrice, true);
  t.end();
});
