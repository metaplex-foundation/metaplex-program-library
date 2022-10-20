import { inspect } from "util";
import { NftWithToken, SftWithToken } from "@metaplex-foundation/js";
import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import {
  createAddCollectionConstraintToEscrowConstraintModelInstruction,
  createAddNoneConstraintToEscrowConstraintModelInstruction,
  createAddTokensConstraintToEscrowConstraintModelInstruction,
  createCreateEscrowConstraintModelAccountInstruction,
  createCreateTrifleAccountInstruction,
  createTransferInInstruction,
  EscrowConstraintModel,
  Trifle,
} from "../../../js/src/generated";
import {
  findEscrowConstraintModelPda,
  findEscrowPda,
  findTriflePda,
} from "./pdas";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { findMetadataPda } from "@metaplex-foundation/js";
import {
  PROGRAM_ADDRESS as TM_PROGRAM_ADDRESS,
} from "@metaplex-foundation/mpl-token-metadata/src/generated";
import { EscrowAuthority } from "./utils";

export const createConstraintModel = async (
  connection: Connection,
  keypair: Keypair,
  name: string,
  schema: string,
) => {
  const escrowConstraintModel = await findEscrowConstraintModelPda(
    keypair.publicKey,
    name,
  );

  const createIX = createCreateEscrowConstraintModelAccountInstruction(
    {
      escrowConstraintModel: escrowConstraintModel[0],
      payer: keypair.publicKey,
      updateAuthority: keypair.publicKey,
    },
    {
      createEscrowConstraintModelAccountArgs: {
        name,
        schemaUri: schema,
      },
    },
  );

  const tx = new Transaction().add(createIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = keypair.publicKey;
  await connection.sendTransaction(tx, [keypair], { skipPreflight: true });
  // await connection.sendTransaction(tx, [keypair]);

  return escrowConstraintModel[0];
};

export const addNoneConstraint = async (
  connection: Connection,
  keypair: Keypair,
  name: string,
  tokenLimit: number,
  transferEffects: number,
  model: PublicKey,
) => {
  const addIX = createAddNoneConstraintToEscrowConstraintModelInstruction(
    {
      escrowConstraintModel: model,
      payer: keypair.publicKey,
      updateAuthority: keypair.publicKey,
    },
    {
      addNoneConstraintToEscrowConstraintModelArgs: {
        constraintName: name,
        tokenLimit: tokenLimit,
        transferEffects,
      },
    },
  );

  const tx = new Transaction().add(addIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = keypair.publicKey;
  await connection.sendTransaction(tx, [keypair], { skipPreflight: true });
};

export const addCollectionConstraint = async (
  connection: Connection,
  keypair: Keypair,
  name: string,
  tokenLimit: number,
  collection: PublicKey,
  transferEffects: number,
  model: PublicKey,
) => {
  const collectionMintMetadata = await findMetadataPda(collection);
  const addIX = createAddCollectionConstraintToEscrowConstraintModelInstruction(
    {
      escrowConstraintModel: model,
      payer: keypair.publicKey,
      updateAuthority: keypair.publicKey,
      collectionMint: collection,
      collectionMintMetadata,
    },
    {
      addCollectionConstraintToEscrowConstraintModelArgs: {
        constraintName: name,
        tokenLimit: tokenLimit,
        transferEffects,
      },
    },
  );

  const tx = new Transaction().add(addIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = keypair.publicKey;
  await connection.sendTransaction(tx, [keypair], { skipPreflight: true });
};

export const addTokensConstraint = async (
  connection: Connection,
  keypair: Keypair,
  name: string,
  tokenLimit: number,
  tokens: PublicKey[],
  transferEffects: number,
  model: PublicKey,
) => {
  const addIX = createAddTokensConstraintToEscrowConstraintModelInstruction(
    {
      escrowConstraintModel: model,
      payer: keypair.publicKey,
      updateAuthority: keypair.publicKey,
    },
    {
      addTokensConstraintToEscrowConstraintModelArgs: {
        constraintName: name,
        tokenLimit: tokenLimit,
        tokens,
        transferEffects,
      },
    },
  );

  const tx = new Transaction().add(addIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = keypair.publicKey;
  await connection.sendTransaction(tx, [keypair], { skipPreflight: true });
};

export const createTrifle = async (
  connection: Connection,
  nft: NftWithToken,
  keypair: Keypair,
  model_name: string,
) => {
  const escrowConstraintModel = await findEscrowConstraintModelPda(
    keypair.publicKey,
    model_name,
  );
  const trifleAddress = await findTriflePda(
    nft.mint.address,
    keypair.publicKey,
    escrowConstraintModel[0],
  );
  const escrowAccountAddress = await findEscrowPda(
    nft.mint.address,
    EscrowAuthority.Creator,
    trifleAddress[0],
  );

  const createIX = createCreateTrifleAccountInstruction({
    escrow: escrowAccountAddress[0],
    metadata: nft.metadataAddress,
    mint: nft.mint.address,
    tokenAccount: nft.token.address,
    edition: nft.edition.address,
    trifleAccount: trifleAddress[0],
    trifleAuthority: keypair.publicKey,
    escrowConstraintModel: escrowConstraintModel[0],
    payer: keypair.publicKey,
    tokenMetadataProgram: new PublicKey(TM_PROGRAM_ADDRESS),
  });

  const tx = new Transaction().add(createIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = keypair.publicKey;
  const sig = await connection.sendTransaction(tx, [keypair], {
    skipPreflight: false,
  });
  await connection.confirmTransaction(sig, "finalized");

  return trifleAddress[0];
};

export const transferIn = async (
  connection: Connection,
  escrowNft: NftWithToken,
  escrowAccountAddress: PublicKey,
  nft: NftWithToken | SftWithToken,
  keypair: Keypair,
  slot: string,
) => {
  const escrowConstraintModel = await findEscrowConstraintModelPda(
    keypair.publicKey,
    "test",
  );
  const trifleAddress = await findTriflePda(
    escrowNft.mint.address,
    keypair.publicKey,
    escrowConstraintModel[0],
  );

  const dst: PublicKey = await getAssociatedTokenAddress(
    nft.mint.address,
    escrowAccountAddress,
    true,
  );

  // trifle: web3.PublicKey;
  // trifleAuthority: web3.PublicKey;
  // payer: web3.PublicKey;
  // constraintModel: web3.PublicKey;
  // escrow: web3.PublicKey;
  // escrowMint?: web3.PublicKey;
  // escrowToken?: web3.PublicKey;
  // escrowEdition?: web3.PublicKey;
  // attributeMint?: web3.PublicKey;
  // attributeSrcToken?: web3.PublicKey;
  // attributeDstToken?: web3.PublicKey;
  // attributeMetadata?: web3.PublicKey;
  // attributeEdition?: web3.PublicKey;
  // attributeCollectionMetadata?: web3.PublicKey;
  const transferIX = createTransferInInstruction(
    {
      trifle: trifleAddress[0],
      constraintModel: escrowConstraintModel[0],
      escrow: escrowAccountAddress,
      payer: keypair.publicKey,
      trifleAuthority: keypair.publicKey,
      attributeMint: nft.mint.address,
      attributeSrcToken: nft.token.address,
      attributeDstToken: dst,
      attributeMetadata: nft.metadataAddress,
      escrowMint: escrowNft.mint.address,
      escrowToken: escrowNft.token.address,
    },
    {
      transferInArgs: { amount: 1, slot },
    },
  );

  const tx = new Transaction().add(transferIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = keypair.publicKey;
  console.log(tx);
  const sig = await connection.sendTransaction(tx, [keypair], {
    skipPreflight: true,
  });
  await connection.confirmTransaction(sig, "finalized");
};

export const showModel = async (
  connection: Connection,
  modelAddress: PublicKey,
) => {
  // console.log("Fetching " + modelAddress.toString());
  const accountInfo = await connection.getAccountInfo(modelAddress);
  if (accountInfo) {
    const account: EscrowConstraintModel =
      EscrowConstraintModel.fromAccountInfo(accountInfo)[0];
    // console.log(JSON.stringify(account.pretty()));
    console.log(JSON.stringify(account));
    let inspected = inspect(account.constraints);
    console.log(inspected);
  } else {
    console.log("Unable to fetch account");
  }
};

export const showTrifle = async (
  connection: Connection,
  trifleAddress: PublicKey,
) => {
  // console.log("Fetching " + trifleAddress.toString());
  const accountInfo = await connection.getAccountInfo(trifleAddress);
  if (accountInfo) {
    const account: Trifle = Trifle.fromAccountInfo(accountInfo)[0];
    console.log(JSON.stringify(account.pretty()));
  } else {
    console.log("Unable to fetch account");
  }
};
