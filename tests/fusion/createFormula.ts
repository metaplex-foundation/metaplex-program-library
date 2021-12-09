import * as anchor from "@project-serum/anchor";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  AccountMeta,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { assert, expect } from "chai";
import {
  createIngredientMints,
  createIngredients,
  initNewTokenMint,
  processOutputItemsForCreateFormula,
  setupMetaplexMasterEdition,
} from "./utils";
import { BN } from "@project-serum/anchor";
import { Formula, Ingredient, Item } from "./types";

const textEncoder = new TextEncoder();

describe("create_formula", () => {
  const provider = anchor.Provider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.Fusion;
  const payer = anchor.web3.Keypair.generate();

  // The mintAuthority for the ingredients (2-to-1 crafting)
  const mintAuthority = anchor.web3.Keypair.generate();
  let ingredientMintA: PublicKey,
    ingredientMintB: PublicKey,
    outputMint: PublicKey;

  // The mintAuthority for the ingredients (4-to-6 crafting)
  const mintAuthorityOne = anchor.web3.Keypair.generate();
  let ingredientMintOne: PublicKey,
    ingredientMintTwo: PublicKey,
    ingredientMintThree: PublicKey,
    ingredientMintFour: PublicKey;
  let outputMintOne: PublicKey,
    outputMintTwo: PublicKey,
    outputMintThree: PublicKey,
    outputMintFour: PublicKey,
    outputMintFive: PublicKey,
    outputMintSix: PublicKey;

  before(async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10_000_000_000),
      "confirmed"
    );
  });

  describe("Two to one crafting", () => {
    before(async () => {
      // create the initial 2 mints (not owned by the user)
      [ingredientMintA, ingredientMintB] = await createIngredientMints(
        provider.connection,
        mintAuthority.publicKey,
        payer,
        2
      );
      // create the 1 output mint which is owned by the user
      const { mintAccount } = await initNewTokenMint(
        provider.connection,
        payer.publicKey,
        payer,
        0
      );
      outputMint = mintAccount.publicKey;
    });

    it("should create a Formula and transfer the mint authority for output items", async () => {
      const ingredients = [
        {
          mint: ingredientMintA,
          amount: 1,
          burnOnCraft: true,
        },
        {
          mint: ingredientMintB,
          amount: 1,
          burnOnCraft: true,
        },
      ];
      const outputItems: Item[] = [
        {
          mint: outputMint,
          amount: 1,
          isMasterEdition: false,
        },
      ];

      const remainingAccounts: AccountMeta[] = outputItems.map((x) => ({
        pubkey: x.mint,
        isWritable: true,
        isSigner: false,
      }));
      const expectedFormula = {
        ingredients,
        outputItems,
      };
      // Generate new keypair for the Formula account
      const formulaKeypair = anchor.web3.Keypair.generate();

      const [craftingMintAuthority, craftingMintAuthorityBump] =
        await PublicKey.findProgramAddress(
          [textEncoder.encode("crafting"), formulaKeypair.publicKey.toBuffer()],
          program.programId
        );

      await program.rpc.createFormula(
        expectedFormula.ingredients,
        expectedFormula.outputItems,
        craftingMintAuthorityBump,
        {
          accounts: {
            formula: formulaKeypair.publicKey,
            authority: payer.publicKey,
            outputAuthority: craftingMintAuthority,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          },
          remainingAccounts,
          signers: [payer, formulaKeypair],
        }
      );

      // Validate the Formula gets created and stored on chain properly
      const formula = await program.account.formula.fetch(
        formulaKeypair.publicKey
      );
      expect(formula.ingredients).to.eql(expectedFormula.ingredients);
      const output = formula.outputItems.map((x: Item) => {
        delete x.masterTokenAccount;
        return x;
      });
      expect(output).to.eql(expectedFormula.outputItems);

      // Vaidate the mint authority for the output items gets transfered to the formula
      await Promise.all(
        expectedFormula.outputItems.map(async (outputItem) => {
          const token = new Token(
            provider.connection,
            outputItem.mint,
            TOKEN_PROGRAM_ID,
            payer
          );
          const outputMintAfter = await token.getMintInfo();
          assert.ok(
            outputMintAfter.mintAuthority?.equals(craftingMintAuthority)
          );
        })
      );
    });
  });

  describe("Four to six crafting", () => {
    before(async () => {
      // create the initial 4 mints (not owned by the user)
      [
        ingredientMintOne,
        ingredientMintTwo,
        ingredientMintThree,
        ingredientMintFour,
      ] = await createIngredientMints(
        provider.connection,
        mintAuthorityOne.publicKey,
        payer,
        4
      );

      // create the 6 output mint which is owned by the user
      const mintAccountOne = await initNewTokenMint(
        provider.connection,
        payer.publicKey,
        payer,
        0
      ).then((_) => _.mintAccount);
      outputMintOne = mintAccountOne.publicKey;
      const mintAccountTwo = await initNewTokenMint(
        provider.connection,
        payer.publicKey,
        payer,
        0
      ).then((_) => _.mintAccount);
      outputMintTwo = mintAccountTwo.publicKey;
      const mintAccountThree = await initNewTokenMint(
        provider.connection,
        payer.publicKey,
        payer,
        0
      ).then((_) => _.mintAccount);
      outputMintThree = mintAccountThree.publicKey;
      const mintAccountFour = await initNewTokenMint(
        provider.connection,
        payer.publicKey,
        payer,
        0
      ).then((_) => _.mintAccount);
      outputMintFour = mintAccountFour.publicKey;
      const mintAccountFive = await initNewTokenMint(
        provider.connection,
        payer.publicKey,
        payer,
        0
      ).then((_) => _.mintAccount);
      outputMintFive = mintAccountFive.publicKey;
      const mintAccountSix = await initNewTokenMint(
        provider.connection,
        payer.publicKey,
        payer,
        0
      ).then((_) => _.mintAccount);
      outputMintSix = mintAccountSix.publicKey;
    });

    it("should create a Formula and transfer the mint authority for output items", async () => {
      const ingredients = [
        {
          mint: ingredientMintOne,
          amount: 1,
          burnOnCraft: true,
        },
        {
          mint: ingredientMintTwo,
          amount: 1,
          burnOnCraft: true,
        },
        {
          mint: ingredientMintThree,
          amount: 1,
          burnOnCraft: true,
        },
        {
          mint: ingredientMintFour,
          amount: 1,
          burnOnCraft: true,
        },
      ];

      const outputItems: Item[] = [
        {
          mint: outputMintOne,
          amount: 1,
          isMasterEdition: false,
        },
        {
          mint: outputMintTwo,
          amount: 1,
          isMasterEdition: false,
        },
        {
          mint: outputMintThree,
          amount: 1,
          isMasterEdition: false,
        },
        {
          mint: outputMintFour,
          amount: 1,
          isMasterEdition: false,
        },
        {
          mint: outputMintFive,
          amount: 1,
          isMasterEdition: false,
        },
        {
          mint: outputMintSix,
          amount: 1,
          isMasterEdition: false,
        },
      ];

      const remainingAccounts: AccountMeta[] = outputItems.map((x) => ({
        pubkey: x.mint,
        isWritable: true,
        isSigner: false,
      }));

      const expectedFormula = {
        ingredients,
        outputItems,
      };

      // Generate new keypair for the Formula account
      const formulaKeypair = anchor.web3.Keypair.generate();

      const [craftingMintAuthority, craftingMintAuthorityBump] =
        await PublicKey.findProgramAddress(
          [textEncoder.encode("crafting"), formulaKeypair.publicKey.toBuffer()],
          program.programId
        );

      await program.rpc.createFormula(
        expectedFormula.ingredients,
        expectedFormula.outputItems,
        craftingMintAuthorityBump,
        {
          accounts: {
            formula: formulaKeypair.publicKey,
            authority: payer.publicKey,
            outputAuthority: craftingMintAuthority,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          },
          remainingAccounts,
          signers: [payer, formulaKeypair],
        }
      );

      // Validate the Formula gets created and stored on chain properly
      const formula = await program.account.formula.fetch(
        formulaKeypair.publicKey
      );
      expect(formula).to.eql(expectedFormula);

      // Vaidate the mint authority for the output items gets transfered to the formula
      await Promise.all(
        expectedFormula.outputItems.map(async (outputItem) => {
          const token = new Token(
            provider.connection,
            outputItem.mint,
            TOKEN_PROGRAM_ID,
            payer
          );
          const outputMintAfter = await token.getMintInfo();
          assert.ok(
            outputMintAfter.mintAuthority?.equals(craftingMintAuthority)
          );
        })
      );
    });
  });

  describe("Single output as a metaplex Edition", () => {
    let masterToken: Token,
      masterEditionHolder: PublicKey,
      ingredientMintA: PublicKey,
      ingredientMintB: PublicKey;
    beforeEach(async () => {
      // Prior to creating the formula, a user must interact with the Token-Metadata contract
      //  to create MasterEditions for all the outputs
      const { masterEditionHolder: _masterEditionHolder, masterTokenKey } =
        await setupMetaplexMasterEdition(provider);
      masterEditionHolder = _masterEditionHolder;
      masterToken = new Token(
        provider.connection,
        masterTokenKey,
        TOKEN_PROGRAM_ID,
        payer
      );
      // Create ingredient mints
      [ingredientMintA, ingredientMintB] = await createIngredientMints(
        provider.connection,
        mintAuthority.publicKey,
        payer,
        2
      );
    });

    it("should create new Formula with the output mint", async () => {
      const ingredients: Ingredient[] = createIngredients(
        [ingredientMintA, ingredientMintB],
        [1, 1],
        true
      );
      const outputItems: Item[] = [
        {
          mint: masterToken.publicKey,
          amount: 1,
          isMasterEdition: true,
        },
      ];
      const formula: Formula = {
        ingredients,
        outputItems,
      };

      // Generate new keypair for the Formula account
      const formulaKeypair = anchor.web3.Keypair.generate();

      const [craftingMintAuthority, craftingMintAuthorityBump] =
        await PublicKey.findProgramAddress(
          [textEncoder.encode("crafting"), formulaKeypair.publicKey.toBuffer()],
          program.programId
        );

      const remainingAccounts: AccountMeta[] = [],
        masterTokenAccounts: PublicKey[] = [];
      await processOutputItemsForCreateFormula(
        program,
        formulaKeypair.publicKey,
        formula.outputItems,
        [masterEditionHolder],
        remainingAccounts,
        masterTokenAccounts
      );

      try {
        await program.rpc.createFormula(
          formula.ingredients,
          formula.outputItems,
          craftingMintAuthorityBump,
          {
            accounts: {
              formula: formulaKeypair.publicKey,
              authority: provider.wallet.publicKey,
              outputAuthority: craftingMintAuthority,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
              rent: SYSVAR_RENT_PUBKEY,
            },
            remainingAccounts,
            signers: [formulaKeypair],
          }
        );
      } catch (err) {
        console.error(err);
        throw err;
      }

      assert.ok(true);

      // Validate that the program now controls the MasterEdition token
      await Promise.all(
        masterTokenAccounts.map(async (masterTokenAccount) => {
          const programMasterTokenInfo = await masterToken.getAccountInfo(
            masterTokenAccount
          );
          assert.ok(programMasterTokenInfo.amount.eqn(1));
        })
      );

      // Validate that the original masterEditionHolder no longer has it
      const oldHolerInfo = await masterToken.getAccountInfo(
        masterEditionHolder
      );
      assert.ok(oldHolerInfo.amount.eqn(0));
    });
  });
});
