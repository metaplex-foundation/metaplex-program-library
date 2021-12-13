import * as anchor from "@project-serum/anchor";
import { BN } from "@project-serum/anchor";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  AccountMeta,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { assert } from "chai";
import { decodeMasterEdition } from "./metadata_utils";
import { Formula, Ingredient, Item } from "./types";
import {
  createIngredientMints,
  initNewTokenMint,
  createIngredients,
  createOutputItems,
  createAssociatedTokenAccount,
  mintTokensToAccount,
  setupMetaplexMasterEdition,
  createAccountsForOutputPrint,
  processOutputItemsForCreateFormula,
  deriveMasterTokenAccount,
} from "./utils";

const textEncoder = new TextEncoder();

describe("craft", async () => {
  const provider = anchor.Provider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.Fusion;
  // Account that pays to create the formula
  const payer = anchor.web3.Keypair.generate();
  // Account that crafts the formula using their token accounts
  const crafter = anchor.web3.Keypair.generate();

  // The mintAuthority for the formula's ingredients - 2-to-1 formula
  const mintAuthority = anchor.web3.Keypair.generate();
  let ingredientMintA: PublicKey,
    ingredientMintB: PublicKey,
    outputMint: PublicKey,
    outputToken: Token;
  let formulaKp: Keypair;

  before(async () => {
    // Airdrop lamports to payer & crafter
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10_000_000_000),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        crafter.publicKey,
        10_000_000_000
      ),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        mintAuthority.publicKey,
        10_000_000_000
      ),
      "confirmed"
    );
  });

  describe("SPL based formula", () => {
    // Set up a formula to craft in main test block
    before("Create a 2-to-1 formula", async () => {
      // Create ingredient mints
      [ingredientMintA, ingredientMintB] = await createIngredientMints(
        provider.connection,
        mintAuthority.publicKey,
        payer,
        2
      );

      // Create output mint
      const { mintAccount } = await initNewTokenMint(
        provider.connection,
        payer.publicKey,
        payer,
        0
      );
      outputMint = mintAccount.publicKey;
      outputToken = new Token(
        provider.connection,
        outputMint,
        TOKEN_PROGRAM_ID,
        payer
      );

      const ingredients = createIngredients(
        [ingredientMintA, ingredientMintB],
        [1, 1],
        true
      );
      const outputItems = createOutputItems([outputMint], [1]);

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

      debugger;

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
      formulaKp = formulaKeypair;
    });

    it("Craft the formula", async () => {
      // grab the formula from the chain
      const formula = (await program.account.formula.fetch(
        formulaKp.publicKey
      )) as Formula;

      let remainingAccounts: AccountMeta[] = [];
      let ingredientTokenPubkeys: PublicKey[] = [];
      let outputTokenPubkeys: PublicKey[] = [];

      // Create ingredient ATAs for crafter
      const starterPromise = Promise.resolve(null);
      await formula.ingredients.reduce(async (accumulator, ingredient) => {
        await accumulator;
        let craftersTokenAccount = await createAssociatedTokenAccount(
          provider.connection,
          crafter,
          ingredient.mint
        );

        // Mint the right amount of ingredient tokens to the crafter's ATAs
        await mintTokensToAccount(
          provider.connection,
          ingredient.amount,
          ingredient.mint,
          craftersTokenAccount,
          mintAuthority
        );

        remainingAccounts.push({
          pubkey: craftersTokenAccount,
          isWritable: ingredient.burnOnCraft,
          isSigner: false,
        });
        ingredientTokenPubkeys.push(craftersTokenAccount);

        // Push ingredient mints
        remainingAccounts.push({
          pubkey: ingredient.mint,
          isWritable: ingredient.burnOnCraft,
          isSigner: false,
        });
        return null;
      }, starterPromise);

      const [outMintPda, outBump] = await PublicKey.findProgramAddress(
        [textEncoder.encode("crafting"), formulaKp.publicKey.toBuffer()],
        program.programId
      );

      await formula.outputItems.reduce(async (accumulator, item) => {
        await accumulator;
        // Create output item ATAs for crafter
        const outputItemTokenAccount = await createAssociatedTokenAccount(
          provider.connection,
          crafter,
          item.mint
        );

        // Push output item token account
        remainingAccounts.push({
          pubkey: outputItemTokenAccount,
          isWritable: true,
          isSigner: false,
        });
        outputTokenPubkeys.push(outputItemTokenAccount);

        // Push output Item mints
        remainingAccounts.push({
          pubkey: item.mint,
          isWritable: true,
          isSigner: false,
        });
        return null;
      }, starterPromise);

      try {
        await program.rpc.craft(outBump, {
          accounts: {
            authority: crafter.publicKey,
            formula: formulaKp.publicKey,
            pdaAuth: outMintPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          },
          remainingAccounts,
          signers: [crafter],
        });
      } catch (err) {
        console.error(err);
        throw err;
      }
      assert.ok(true);

      // Query and assert that ingredient token balances are 0
      const ingredientPromiseStart = Promise.resolve(null);
      await ingredientTokenPubkeys.reduce(async (accumulator, pubkey) => {
        await accumulator;

        const balance = await provider.connection.getTokenAccountBalance(
          pubkey
        );
        assert.ok("0" == balance.value.amount);

        return null;
      }, ingredientPromiseStart);

      // Query and assert that output token balances are as formula describes
      const outoutPromiseStart = Promise.resolve(null);
      await outputTokenPubkeys.reduce(async (accumulator, output, index) => {
        await accumulator;

        const balance = await provider.connection.getTokenAccountBalance(
          output
        );
        assert.ok(
          formula.outputItems[index].amount.toString() == balance.value.amount
        );

        return null;
      }, outoutPromiseStart);
    });
  });

  describe("2-to-1 Metaplex print output formula", () => {
    let masterEditionKey: PublicKey,
      masterTokenMintKey: PublicKey,
      formula: Formula;
    beforeEach(async () => {
      // create the metaplex output info
      const { masterEditionHolder, masterTokenKey, editionAccount } =
        await setupMetaplexMasterEdition(provider);
      masterEditionKey = editionAccount;
      masterTokenMintKey = masterTokenKey;
      // Create ingredient mints
      [ingredientMintA, ingredientMintB] = await createIngredientMints(
        provider.connection,
        mintAuthority.publicKey,
        payer,
        2
      );
      // TODO: Clean up the copy pasta for creating the Formula
      const ingredients: Ingredient[] = createIngredients(
        [ingredientMintA, ingredientMintB],
        [1, 1],
        true
      );
      const outputItems: Item[] = [
        {
          mint: masterTokenKey,
          amount: 1,
          isMasterEdition: true,
        },
      ];
      formula = {
        ingredients,
        outputItems,
      };

      // Generate new keypair for the Formula account
      formulaKp = anchor.web3.Keypair.generate();

      const [craftingMintAuthority, craftingMintAuthorityBump] =
        await PublicKey.findProgramAddress(
          [textEncoder.encode("crafting"), formulaKp.publicKey.toBuffer()],
          program.programId
        );

      const remainingAccounts: AccountMeta[] = [],
        masterTokenAccounts: PublicKey[] = [];
      await processOutputItemsForCreateFormula(
        program,
        formulaKp.publicKey,
        formula.outputItems,
        [masterEditionHolder],
        remainingAccounts,
        masterTokenAccounts
      );

      await program.rpc.createFormula(
        formula.ingredients,
        formula.outputItems,
        craftingMintAuthorityBump,
        {
          accounts: {
            formula: formulaKp.publicKey,
            authority: provider.wallet.publicKey,
            outputAuthority: craftingMintAuthority,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          },
          remainingAccounts,
          signers: [formulaKp],
        }
      );
    });

    it("should craft a new print", async () => {
      // Get and deserialize the master edition info
      const masterEditionInfoBefore = await provider.connection.getAccountInfo(
        masterEditionKey
      );
      if (!masterEditionInfoBefore) {
        throw new Error("masterEditionInfoBefore account does not exist");
      }
      const masterEditionBefore = decodeMasterEdition(
        masterEditionInfoBefore.data
      );
      assert.ok(masterEditionBefore.supply.eqn(0));

      let remainingAccounts: AccountMeta[] = [];
      let ingredientTokenPubkeys: PublicKey[] = [];
      let outputTokenPubkeys: PublicKey[] = [];

      // Create ingredient ATAs for crafter
      const starterPromise = Promise.resolve(null);
      await formula.ingredients.reduce(async (accumulator, ingredient) => {
        await accumulator;
        let craftersTokenAccount = await createAssociatedTokenAccount(
          provider.connection,
          crafter,
          ingredient.mint
        );

        // Mint the right amount of ingredient tokens to the crafter's ATAs
        await mintTokensToAccount(
          provider.connection,
          ingredient.amount,
          ingredient.mint,
          craftersTokenAccount,
          mintAuthority
        );

        remainingAccounts.push({
          pubkey: craftersTokenAccount,
          isWritable: ingredient.burnOnCraft,
          isSigner: false,
        });
        ingredientTokenPubkeys.push(craftersTokenAccount);

        // Push ingredient mints
        remainingAccounts.push({
          pubkey: ingredient.mint,
          isWritable: ingredient.burnOnCraft,
          isSigner: false,
        });
        return null;
      }, starterPromise);

      const [craftingMintAuthority, craftingMintAuthorityBump] =
        await PublicKey.findProgramAddress(
          [textEncoder.encode("crafting"), formulaKp.publicKey.toBuffer()],
          program.programId
        );

      await formula.outputItems.reduce(async (accumulator, item) => {
        await accumulator;
        if (item.isMasterEdition) {
          // Derive the masterTokenKey
          const [masterTokenAccount] = await deriveMasterTokenAccount(
            formulaKp.publicKey,
            item.mint,
            program.programId
          );
          // add the remaining accounts for the master edition print
          const outputPrintAccounts = await createAccountsForOutputPrint(
            provider,
            masterTokenMintKey,
            craftingMintAuthority,
            masterTokenAccount,
            craftingMintAuthority,
            item,
            new BN(1)
          );
          remainingAccounts = [...remainingAccounts, ...outputPrintAccounts];
        } else {
          // Create output item ATAs for crafter
          const outputItemTokenAccount = await createAssociatedTokenAccount(
            provider.connection,
            crafter,
            item.mint
          );

          // Push output item token account
          remainingAccounts.push({
            pubkey: outputItemTokenAccount,
            isWritable: true,
            isSigner: false,
          });
          outputTokenPubkeys.push(outputItemTokenAccount);

          // Push output Item mints
          remainingAccounts.push({
            pubkey: item.mint,
            isWritable: true,
            isSigner: false,
          });
        }

        return null;
      }, starterPromise);

      try {
        await program.rpc.craft(craftingMintAuthorityBump, {
          accounts: {
            authority: crafter.publicKey,
            formula: formulaKp.publicKey,
            pdaAuth: craftingMintAuthority,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          },
          remainingAccounts,
          signers: [crafter],
        });
      } catch (err) {
        console.error(err);
        throw err;
      }

      // Check that the master token's metadata edition number incremented
      const masterEditionInfoAfter = await provider.connection.getAccountInfo(
        masterEditionKey
      );
      if (!masterEditionInfoAfter) {
        throw new Error("masterEditionInfoAfter account does not exist");
      }
      const masterEditionAfter = decodeMasterEdition(
        masterEditionInfoAfter.data
      );
      assert.ok(masterEditionAfter.supply.eqn(1));
    });

    describe("bad metaplex metadata program", () => {
      it("should error", async () => {
        let remainingAccounts: AccountMeta[] = [];
        let ingredientTokenPubkeys: PublicKey[] = [];
        let outputTokenPubkeys: PublicKey[] = [];

        // Create ingredient ATAs for crafter
        const starterPromise = Promise.resolve(null);
        await formula.ingredients.reduce(async (accumulator, ingredient) => {
          await accumulator;
          let craftersTokenAccount = await createAssociatedTokenAccount(
            provider.connection,
            crafter,
            ingredient.mint
          );

          // Mint the right amount of ingredient tokens to the crafter's ATAs
          await mintTokensToAccount(
            provider.connection,
            ingredient.amount,
            ingredient.mint,
            craftersTokenAccount,
            mintAuthority
          );

          remainingAccounts.push({
            pubkey: craftersTokenAccount,
            isWritable: ingredient.burnOnCraft,
            isSigner: false,
          });
          ingredientTokenPubkeys.push(craftersTokenAccount);

          // Push ingredient mints
          remainingAccounts.push({
            pubkey: ingredient.mint,
            isWritable: ingredient.burnOnCraft,
            isSigner: false,
          });
          return null;
        }, starterPromise);

        const [craftingMintAuthority, craftingMintAuthorityBump] =
          await PublicKey.findProgramAddress(
            [textEncoder.encode("crafting"), formulaKp.publicKey.toBuffer()],
            program.programId
          );

        await formula.outputItems.reduce(async (accumulator, item) => {
          await accumulator;
          if (item.isMasterEdition) {
            // Derive the masterTokenKey
            const [masterTokenAccount] = await deriveMasterTokenAccount(
              formulaKp.publicKey,
              item.mint,
              program.programId
            );
            // add the remaining accounts for the master edition print
            const outputPrintAccounts = await createAccountsForOutputPrint(
              provider,
              masterTokenMintKey,
              craftingMintAuthority,
              masterTokenAccount,
              craftingMintAuthority,
              item,
              new BN(1)
            );
            outputPrintAccounts[0] = {
              pubkey: SystemProgram.programId,
              isSigner: false,
              isWritable: false,
            };
            remainingAccounts = [...remainingAccounts, ...outputPrintAccounts];
          } else {
            // Create output item ATAs for crafter
            const outputItemTokenAccount = await createAssociatedTokenAccount(
              provider.connection,
              crafter,
              item.mint
            );

            // Push output item token account
            remainingAccounts.push({
              pubkey: outputItemTokenAccount,
              isWritable: true,
              isSigner: false,
            });
            outputTokenPubkeys.push(outputItemTokenAccount);

            // Push output Item mints
            remainingAccounts.push({
              pubkey: item.mint,
              isWritable: true,
              isSigner: false,
            });
          }

          return null;
        }, starterPromise);

        try {
          await program.rpc.craft(craftingMintAuthorityBump, {
            accounts: {
              authority: crafter.publicKey,
              formula: formulaKp.publicKey,
              pdaAuth: craftingMintAuthority,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
              rent: SYSVAR_RENT_PUBKEY,
            },
            remainingAccounts,
            signers: [crafter],
          });
          assert.ok(false);
        } catch (err) {
          const errMsg = "Invalid token metadata program";
          assert.equal((err as Error).toString(), errMsg);
        }
      });
    });
  });
});
