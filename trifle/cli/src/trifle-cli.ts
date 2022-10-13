#!/usr/bin/env node

import chalk from "chalk";
import clear from "clear";
import * as figlet from "figlet";
import * as path from "path";
import { program } from "commander";
import log from "loglevel";
import * as sdk from "@metaplex-foundation/mpl-token-metadata/src/generated";
import * as web3 from "@solana/web3.js";
import * as fs from "fs";
import { Keypair } from "@solana/web3.js";
import {
  keypairIdentity,
  Metaplex,
  Nft,
  NftWithToken,
  Sft,
  SftWithToken,
} from "@metaplex-foundation/js";
import { use_metaplex } from "./helpers/utils";
import {
  addCollectionConstraint,
  addNoneConstraint,
  addTokensConstraint,
  createConstraintModel,
  createTrifle,
  showModel,
  showTrifle,
} from "./helpers/trifle";
import { findEscrowConstraintModelPda, findTriflePda } from "./helpers/pdas";
import { Key } from "@metaplex-foundation/mpl-token-metadata";
import { PublicKeyMismatchError } from "@metaplex-foundation/mpl-auction-house";

// TODO: show this on -h or --help
// clear();
// console.log(
//   chalk.green(figlet.textSync("Trifle CLI", { horizontalLayout: "full" })),
// );

const create = program.command("create");

create
  .command("model")
  .option(
    "-e, --env <string>",
    "Solana cluster env name",
    "devnet", //mainnet-beta, testnet, devnet
  )
  .option("-r, --rpc <string>", "The endpoint to connect to.")
  .option(
    "-k, --keypair <path>",
    `Solana wallet location`,
  )
  .option("-l, --log-level <string>", "log level", setLogLevel)
  .option("-n, --name <string>", "The name of the constraint model.")
  .option("-s, --schema <string>", "The schema of the constraint model.")
  .action(async (directory, cmd) => {
    // console.log(cmd.opts());
    const { keypair, env, rpc, name, schema } = cmd.opts();

    const metaplex = await use_metaplex(keypair, env, rpc);

    const modelAddr = await createConstraintModel(
      metaplex.connection,
      new Keypair({
        publicKey: metaplex.identity().publicKey.toBuffer(),
        secretKey: metaplex.identity().secretKey as Uint8Array,
      }),
      name,
      schema,
    );

    // console.log("Constraint Model Created!");
    await showModel(metaplex.connection, modelAddr);
  });

create
  .command("trifle")
  .option(
    "-e, --env <string>",
    "Solana cluster env name",
    "devnet", //mainnet-beta, testnet, devnet
  )
  .option("-r, --rpc <string>", "The endpoint to connect to.")
  .option(
    "-k, --keypair <path>",
    `Solana wallet location`,
  )
  .option("-l, --log-level <string>", "log level", setLogLevel)
  .option(
    "-m, --mint <string>",
    "The mint of the NFT you want to create a trifle for.",
  )
  .option("-c, --create", "Create a new base NFT with the Trifle.")
  .option("-u, --uri <string>", "The URI if creating a new NFT.")
  .option("-n, --name <string>", "The name if creating a new NFT.")
  .option(
    "-mn, --model-name <string>",
    "The name of the constraint model (Assumes keypair is the same as the Model Authority).",
  )
  .action(async (directory, cmd) => {
    // console.log(cmd.opts());
    const { keypair, env, rpc, mint, create, uri, name, modelName } = cmd
      .opts();

    const metaplex = await use_metaplex(keypair, env, rpc);

    let nft: NftWithToken | Nft | SftWithToken | Sft;
    if (create) {
      // Create a new base NFT.
      const { nft: newNFT } = await metaplex
        .nfts()
        .create({
          uri,
          name,
          sellerFeeBasisPoints: 500, // Represents 5.00%.
        })
        .run();
      nft = newNFT;
    } else {
      nft = await metaplex.nfts().findByMint(mint).run();
    }

    const trifleAddr = await createTrifle(
      metaplex.connection,
      nft as NftWithToken,
      new Keypair({
        publicKey: metaplex.identity().publicKey.toBuffer(),
        secretKey: metaplex.identity().secretKey as Uint8Array,
      }),
      modelName,
    );

    // console.log("Trifle Created!");
    await showTrifle(metaplex.connection, trifleAddr);
  });

const constraintCommand = create.command("constraint");

constraintCommand
  .command("none")
  .option(
    "-e, --env <string>",
    "Solana cluster env name",
    "devnet", //mainnet-beta, testnet, devnet
  )
  .option("-r, --rpc <string>", "The endpoint to connect to.")
  .option(
    "-k, --keypair <path>",
    `Solana wallet location`,
  )
  .option("-l, --log-level <string>", "log level", setLogLevel)
  .option("-mn, --model-name <string>", "The name of the constraint model.")
  .option("-cn --constraint-name <string>", "The name of the constraint")
  .option(
    "-l --token-limit <int>",
    "The max number of tokens that can be transferred into this constraint slot",
  )
  .action(async (directory, cmd) => {
    // console.log(cmd.opts());
    const {
      keypair,
      env,
      rpc,
      name,
      schema,
      constraintName,
      modelName,
      tokenLimit,
    } = cmd.opts();

    const metaplex = await use_metaplex(keypair, env, rpc);
    let [modelAddress] = await findEscrowConstraintModelPda(
      metaplex.identity().publicKey,
      modelName,
    );

    const adaptedKeypair = new Keypair({
      publicKey: metaplex.identity().publicKey.toBuffer(),
      secretKey: metaplex.identity().secretKey as Uint8Array,
    });

    await addNoneConstraint(
      metaplex.connection,
      adaptedKeypair,
      constraintName,
      tokenLimit,
      modelAddress,
    );

    await showModel(metaplex.connection, modelAddress);
  });

constraintCommand
  .command("collection")
  .option(
    "-e, --env <string>",
    "Solana cluster env name",
    "devnet", //mainnet-beta, testnet, devnet
  )
  .option("-r, --rpc <string>", "The endpoint to connect to.")
  .option(
    "-k, --keypair <path>",
    `Solana wallet location`,
  )
  .option("-l, --log-level <string>", "log level", setLogLevel)
  .option("-mn, --model-name <string>", "The name of the constraint model.")
  .option("-cn --constraint-name <string>", "The name of the constraint")
  .option(
    "-l --token-limit <int>",
    "The max number of tokens that can be transferred into this constraint slot",
  )
  .option("-c --collection <string>", "The collection address")
  .action(async (directory, cmd) => {
    // console.log(cmd.opts());
    const {
      keypair,
      env,
      rpc,
      constraintName,
      collection,
      modelName,
      tokenLimit,
    } = cmd.opts();

    const metaplex = await use_metaplex(keypair, env, rpc);
    let [modelAddress] = await findEscrowConstraintModelPda(
      metaplex.identity().publicKey,
      modelName,
    );

    const adaptedKeypair = new Keypair({
      publicKey: metaplex.identity().publicKey.toBuffer(),
      secretKey: metaplex.identity().secretKey as Uint8Array,
    });

    let collectionMint = new web3.PublicKey(collection);

    await addCollectionConstraint(
      metaplex.connection,
      adaptedKeypair,
      constraintName,
      tokenLimit,
      collectionMint,
      modelAddress,
    );

    await showModel(metaplex.connection, modelAddress);
  });

constraintCommand
  .command("tokens")
  .option(
    "-e, --env <string>",
    "Solana cluster env name",
    "devnet", //mainnet-beta, testnet, devnet
  )
  .option("-r, --rpc <string>", "The endpoint to connect to.")
  .option(
    "-k, --keypair <path>",
    `Solana wallet location`,
  )
  .option("-l, --log-level <string>", "log level", setLogLevel)
  .option("-mn, --model-name <string>", "The name of the constraint model.")
  .option("-cn --constraint-name <string>", "The name of the constraint")
  .option(
    "-l --token-limit <int>",
    "The max number of tokens that can be transferred into this constraint slot",
  )
  .option(
    "-p --token-file-path <path>",
    "The path to the file containing the tokens. Should contain a top-level array of token mint addresses.",
  )
  .action(async (directory, cmd) => {
    // console.log(cmd.opts());
    const {
      keypair,
      env,
      rpc,
      constraintName,
      collection,
      modelName,
      tokenLimit,
      tokenFilePath,
    } = cmd.opts();

    // console.log(tokenFilePath);
    if (!tokenFilePath) {
      console.error("No token file path provided");
      process.exit(1);
    }

    let tokens: web3.PublicKey[] = [];

    try {
      const data = fs.readFileSync(tokenFilePath, "utf8");
      tokens = JSON.parse(data).map((t: string) => new web3.PublicKey(t));
    } catch (e) {
      console.error("Error reading file: ", e);
      process.exit(1);
    }

    const metaplex = await use_metaplex(keypair, env, rpc);
    let [modelAddress] = await findEscrowConstraintModelPda(
      metaplex.identity().publicKey,
      modelName,
    );

    const adaptedKeypair = new Keypair({
      publicKey: metaplex.identity().publicKey.toBuffer(),
      secretKey: metaplex.identity().secretKey as Uint8Array,
    });
    // TODO: batch process this.

    await addTokensConstraint(
      metaplex.connection,
      adaptedKeypair,
      constraintName,
      tokenLimit,
      tokens,
      modelAddress,
    );

    await showModel(metaplex.connection, modelAddress);
  });

const show = program.command("show");

show
  .command("model")
  .option(
    "-e, --env <string>",
    "Solana cluster env name",
    "devnet", //mainnet-beta, testnet, devnet
  )
  .option("-r, --rpc <string>", "The endpoint to connect to.")
  .option(
    "-k, --keypair <path>",
    `Solana wallet location`,
  )
  .option("-l, --log-level <string>", "log level", setLogLevel)
  .option("-n, --name <string>", "The name if creating a new NFT.")
  .action(async (directory, cmd) => {
    // console.log(cmd.opts());
    const { keypair, env, rpc, name } = cmd.opts();

    const metaplex = await use_metaplex(keypair, env, rpc);

    const modelAddr = await findEscrowConstraintModelPda(
      metaplex.identity().publicKey,
      name,
    );
    await showModel(metaplex.connection, modelAddr[0]);
  });

show
  .command("trifle")
  .option(
    "-e, --env <string>",
    "Solana cluster env name",
    "devnet", //mainnet-beta, testnet, devnet
  )
  .option("-r, --rpc <string>", "The endpoint to connect to.")
  .option(
    "-k, --keypair <path>",
    `Solana wallet location`,
  )
  .option("-l, --log-level <string>", "log level", setLogLevel)
  .option(
    "-m, --mint <string>",
    "The mint of the NFT you want to view the Trifle for.",
  )
  .option("-mn, --model-name <string>", "The Model the Trifle uses.")
  .option(
    "-mn, --model-name <string>",
    "The name of the constraint model (Assumes keypair is the same as the Model Authority).",
  )
  .action(async (directory, cmd) => {
    // console.log(cmd.opts());
    const { keypair, env, rpc, mint, modelName } = cmd.opts();

    const metaplex = await use_metaplex(keypair, env, rpc);

    const modelAddr = await findEscrowConstraintModelPda(
      metaplex.identity().publicKey,
      modelName,
    );
    const trifleAddr = await findTriflePda(
      new web3.PublicKey(mint),
      metaplex.identity().publicKey,
      modelAddr[0],
    );
    await showTrifle(metaplex.connection, trifleAddr[0]);
  });

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function setLogLevel(value, prev) {
  if (value === undefined || value === null) {
    return;
  }
  log.info("setting the log value to: " + value);
  log.setLevel(value);
}

program
  .version("0.0.1")
  .description("CLI for controlling and managing Trifle accounts.")
  .parse(process.argv);
