import { BN, Program, Provider } from "@project-serum/anchor";
import {
  AccountLayout,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MintLayout,
  Token,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

import {
  AccountMeta,
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Signer,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  createMasterEdition,
  createMetadata,
  Creator,
  Data,
  decodeMetadata,
  getEdition,
  getEditionMarkPda,
  getMetadata,
} from "./metadata_utils";
import { Ingredient, Item } from "./types";

const textEncoder = new TextEncoder();
export const TOKEN_METADATA = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

export const initNewTokenMintInstruction = async (
  connection: Connection,
  /** The mint authority for the new mint account */
  mintAuthority: PublicKey,
  /** The public key that should be signing the TX and paying for the new account's rent */
  payer: PublicKey,
  decimals: number = 8
) => {
  const mintAccount = new Keypair();
  const transaction = new Transaction();
  // Create the Option Mint Account with rent exemption
  // Allocate memory for the account
  const mintRentBalance = await connection.getMinimumBalanceForRentExemption(
    MintLayout.span
  );

  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: mintAccount.publicKey,
      lamports: mintRentBalance,
      space: MintLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );
  transaction.add(
    Token.createInitMintInstruction(
      TOKEN_PROGRAM_ID,
      mintAccount.publicKey,
      decimals,
      mintAuthority,
      null
    )
  );
  return {
    transaction,
    signers: [mintAccount],
    mintAccount,
  };
};

export const initNewTokenMint = async (
  connection: Connection,
  /** The owner for the new mint account */
  owner: PublicKey,
  wallet: Keypair,
  decimals: number = 8
) => {
  const { transaction, signers, mintAccount } =
    await initNewTokenMintInstruction(
      connection,
      owner,
      wallet.publicKey,
      decimals
    );
  await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet, ...signers],
    {
      commitment: "confirmed",
    }
  );
  return {
    mintAccount,
  };
};

export const createIngredientMints = async (
  connection: Connection,
  /** The owner for the new mint account */
  owner: PublicKey,
  wallet: Keypair,
  amount: number = 2
) => {
  const ingredientMints: PublicKey[] = [],
    transaction: Transaction = new Transaction();
  let signers: Signer[] = [];
  await Promise.all(
    Array(amount)
      .fill(0)
      .map(async (x) => {
        const {
          transaction: tx,
          signers: newSigners,
          mintAccount,
        } = await initNewTokenMintInstruction(
          connection,
          owner,
          wallet.publicKey,
          0
        );
        signers = [...signers, ...newSigners];
        ingredientMints.push(mintAccount.publicKey);
        tx.instructions.forEach((ix) => transaction.add(ix));
      })
  );

  await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet, ...signers],
    {
      commitment: "confirmed",
    }
  );
  return ingredientMints;
};

export const createAssociatedTokenAccount = async (
  connection: Connection,
  owner: Keypair,
  mint: PublicKey
) => {
  const transaction = new Transaction();
  const associatedTokenId = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    mint,
    owner.publicKey
  );
  transaction.add(
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mint,
      associatedTokenId,
      owner.publicKey,
      owner.publicKey
    )
  );

  await sendAndConfirmTransaction(connection, transaction, [owner], {
    commitment: "confirmed",
  });

  return associatedTokenId;
};

export const mintTokensToAccount = async (
  connection: Connection,
  amount: number,
  mint: PublicKey,
  recipient: PublicKey,
  mintAuthority: Keypair
) => {
  let transaction = new Transaction();

  transaction.add(
    Token.createMintToInstruction(
      TOKEN_PROGRAM_ID,
      mint,
      recipient,
      mintAuthority.publicKey,
      [],
      amount
    )
  );

  await sendAndConfirmTransaction(connection, transaction, [mintAuthority], {
    commitment: "confirmed",
  });
};

export const createIngredients = (
  mintArray: PublicKey[],
  amountArray: number[],
  burnAll: boolean
) => {
  let ingredients: Ingredient[] = [];

  mintArray.forEach((mint, index) => {
    ingredients.push({
      mint: mint,
      amount: amountArray[index],
      burnOnCraft: burnAll,
    });
  });

  return ingredients;
};

export const createOutputItems = (
  mintArray: PublicKey[],
  amountArray: number[]
) => {
  let outputItems: Item[] = [];

  mintArray.forEach((mint, index) => {
    outputItems.push({
      mint: mint,
      amount: amountArray[index],
      isMasterEdition: false,
    });
  });

  return outputItems;
};

export const createItemMints = async (
  connection: Connection,
  /** The owner for the new mint account */
  owner: PublicKey,
  wallet: Keypair,
  amount: number = 2
) => {
  const itemMints: PublicKey[] = [];
  await Promise.all(
    Array(amount)
      .fill(0)
      .map(async (x) => {
        const { mintAccount } = await initNewTokenMint(
          connection,
          owner,
          wallet,
          0
        );
        itemMints.push(mintAccount.publicKey);
      })
  );
  return itemMints;
};

// Fetch the metaplex metadata account associated with the mint passed in
export const fetchMetadata = async (
  connection: Connection,
  mint: PublicKey
) => {
  let [pda, bump] = await PublicKey.findProgramAddress(
    [
      textEncoder.encode("metadata"),
      TOKEN_METADATA.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA
  );

  const metadata_account = await connection.getAccountInfo(pda);
  return decodeMetadata(metadata_account?.data as Buffer);
};

export const initNewTokenAccountInstructions = async (
  connection: Connection,
  /** The owner for the new TokenAccount */
  owner: PublicKey,
  /** The SPL Token Mint address */
  mint: PublicKey,
  payer: PublicKey
) => {
  const tokenAccount = new Keypair();
  const transaction = new Transaction();

  const assetPoolRentBalance =
    await connection.getMinimumBalanceForRentExemption(AccountLayout.span);

  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: tokenAccount.publicKey,
      lamports: assetPoolRentBalance,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );
  transaction.add(
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      mint,
      tokenAccount.publicKey,
      owner
    )
  );
  return {
    transaction,
    tokenAccount,
    signers: [tokenAccount],
  };
};

export const setupMetaplexMasterEdition = async (provider: Provider) => {
  let masterEditionHolder: PublicKey;
  // Prior to creating the formula, a user must interact with the Token-Metadata contract
  //  to create MasterEditions for all the outputs

  // Create new mint for the output
  const {
    transaction,
    signers,
    mintAccount: outputMint,
  } = await initNewTokenMintInstruction(
    provider.connection,
    provider.wallet.publicKey,
    provider.wallet.publicKey,
    0
  );

  // Instruction to Metaplex's Token-Metadata contract to create a new metadata account
  const instructions: TransactionInstruction[] = [];
  const metadataAccount = await createMetadata(
    new Data({
      symbol: "SYM",
      name: "Name",
      uri: " ".repeat(64), // size of url for arweave
      sellerFeeBasisPoints: 50,
      creators: [
        new Creator({
          address: provider.wallet.publicKey.toString(),
          verified: true,
          share: 100,
        }),
      ],
    }),
    provider.wallet.publicKey,
    outputMint.publicKey,
    provider.wallet.publicKey,
    instructions,
    provider.wallet.publicKey
  );

  // Get and create the associated token account for the holder
  masterEditionHolder = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    outputMint.publicKey,
    provider.wallet.publicKey
  );
  transaction.add(
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      outputMint.publicKey,
      masterEditionHolder,
      provider.wallet.publicKey,
      provider.wallet.publicKey
    )
  );
  // Mint one to the user
  transaction.add(
    Token.createMintToInstruction(
      TOKEN_PROGRAM_ID,
      outputMint.publicKey,
      masterEditionHolder,
      provider.wallet.publicKey,
      [],
      1
    )
  );
  // Instruction to `create_master_edition` on the metadata
  const maxSupply = undefined;
  const { editionAccount } = await createMasterEdition(
    maxSupply !== undefined ? new BN(maxSupply) : undefined,
    outputMint.publicKey,
    provider.wallet.publicKey,
    provider.wallet.publicKey,
    provider.wallet.publicKey,
    instructions
  );
  instructions.forEach((ix) => transaction.add(ix));
  await provider.send(transaction, [...signers]);
  return {
    masterEditionHolder,
    masterTokenKey: outputMint.publicKey,
    editionAccount,
  };
};

export const createAccountsForOutputPrint = async (
  provider: Provider,
  masterMetadataMintKey: PublicKey,
  /** Root account that owns the master mint (This should be the crafting formula's output mint authority) */
  tokenOwner: PublicKey,
  /** The TokenAccount that holds the master mint */
  tokenAccount: PublicKey,
  /** Still not 100% sure whether this is the PublicKey that is creating the new Edition, or the key that has authority to update the metadata */
  newUpdateAuthority: PublicKey,
  item: Item,
  edition: BN
) => {
  const newMintAuthority: PublicKey = provider.wallet.publicKey;
  // First we have to create a Mint account
  const { transaction, signers, mintAccount } =
    await initNewTokenMintInstruction(
      provider.connection,
      newMintAuthority,
      provider.wallet.publicKey,
      0
    );

  // Then we have to create a TokenAccount account for that mint
  const holderAddress = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    mintAccount.publicKey,
    provider.wallet.publicKey
  );
  transaction.add(
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mintAccount.publicKey,
      holderAddress,
      provider.wallet.publicKey,
      provider.wallet.publicKey
    )
  );

  // Then we have to mint 1 to the TokenAccount
  transaction.add(
    Token.createMintToInstruction(
      TOKEN_PROGRAM_ID,
      mintAccount.publicKey,
      holderAddress,
      newMintAuthority,
      [],
      1
    )
  );
  await provider.send(transaction, signers);

  // get metadata for the newMint
  const [newMetadataKey] = await getMetadata(mintAccount.publicKey);
  // get metadata for the masterMint
  const [masterMetadataKey] = await getMetadata(item.mint);
  // get Edition for the newMint
  const [newEdition] = await getEdition(mintAccount.publicKey);
  // get Edition for the master mint
  const [masterEdition] = await getEdition(item.mint);
  // get EditionMarkPDA for the masterMint and the Edition
  const [editionMarkPda] = await getEditionMarkPda(item.mint, edition);
  /* 
    To understand what accounts are needed for each output print, refer to the instruction creation 
    https://docs.rs/spl-token-metadata/0.0.1/src/spl_token_metadata/instruction.rs.html#388-442
    
    TODO: There are some optimizations for duplicate accounts when more than one printed output mint
   */
  const accountMetas: AccountMeta[] = [
    {
      pubkey: TOKEN_METADATA,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: newMetadataKey,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: newEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: masterEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: mintAccount.publicKey,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: editionMarkPda,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: newMintAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: tokenOwner,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: tokenAccount,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: newUpdateAuthority,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: masterMetadataKey,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: masterMetadataMintKey,
      isSigner: false,
      isWritable: false,
    },
  ];
  return accountMetas;
};

export const deriveMasterTokenAccount = (
  formulaKey: PublicKey,
  itemMint: PublicKey,
  programId: PublicKey
) =>
  PublicKey.findProgramAddress(
    [
      formulaKey.toBuffer(),
      itemMint.toBuffer(),
      textEncoder.encode("masterTokenAcct"),
    ],
    programId
  );

export const processOutputItemsForCreateFormula = async (
  program: Program,
  formulaKey: PublicKey,
  outputItems: Item[],
  masterEditionHolders: PublicKey[],
  remainingAccounts: AccountMeta[],
  masterTokenAccounts: PublicKey[]
) => {
  const starterPromise = Promise.resolve(null);
  await outputItems.reduce(async (accumulator, item, index) => {
    await accumulator;
    // Push the output mint
    remainingAccounts.push({
      pubkey: item.mint,
      isWritable: true,
      isSigner: false,
    });

    if (item.isMasterEdition) {
      // If the output is a Metaplex MasterEdition we need to push the TokenAccount holding the current MasterEdition
      remainingAccounts.push({
        pubkey: masterEditionHolders[index],
        isWritable: true,
        isSigner: false,
      });

      const [masterTokenAccount] = await deriveMasterTokenAccount(
        formulaKey,
        item.mint,
        program.programId
      );
      // We also need to push the new TokenAccount that the program controls
      remainingAccounts.push({
        pubkey: masterTokenAccount,
        isWritable: true,
        isSigner: false,
      });
      // Store the master token account so we can test
      masterTokenAccounts.push(masterTokenAccount);
    }
    return null;
  }, starterPromise);
};
