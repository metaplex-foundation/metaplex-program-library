import { Command, program } from "commander";
import log from "loglevel";
import * as fs from "fs";
import { clusterApiUrl, Connection, Keypair } from "@solana/web3.js";
import { Metaplex } from "@metaplex-foundation/js";
import * as dotenv from 'dotenv';

import { getAllFiles, getMintlist, getTraitManifest, getTraitManifestCache, addFilesToTraitManifest, TraitManifest } from "./helpers/parsing";

dotenv.config();

program
  .command("analyze")
  .option(
    '-e, --env <string>',
    'Solana cluster env name',
    'devnet', //mainnet-beta, testnet, devnet
  )
  .option(
    '-r, --rpc <string>',
    "The endpoint to connect to.",
  )
  .option(
    '-k, --keypair <path>',
    `Solana wallet location`,
    '--keypair not provided',
  )
  .option('-l, --log-level <string>', 'log level', setLogLevel)
  .option('-c, --collectionId <string>', 'The collection ID pubkey for the collection NFT')
  .option('-cr, --creator <string>', 'The collection ID pubkey for the collection NFT')
  .action(async (directory, cmd) => {
    const { keypair, env, rpc, collectionId, creator } = cmd.opts();

    const walletKeyPair = loadKeypair(keypair);
    let connection;
    if (rpc !== "") {
      connection = new Connection(rpc);
    }
    else {
      connection = new Connection(clusterApiUrl(env));
    }

    const metaplex = new Metaplex(connection);

    if (collectionId) {
      await getTraitManifest(await getMintlist(metaplex, collectionId, null));
    }
    else if (creator) {
      await getTraitManifest(await getMintlist(metaplex, null, creator));
    }
    else {
      console.log("Please provide either a collectionId or a creator address");
    }
  });

  program
  .command("search")
  .option(
    '-e, --env <string>',
    'Solana cluster env name',
    'devnet', //mainnet-beta, testnet, devnet
  )
  .option(
    '-r, --rpc <string>',
    "The endpoint to connect to.",
  )
  .option(
    '-k, --keypair <path>',
    `Solana wallet location`,
    '--keypair not provided',
  )
  .option('-l, --log-level <string>', 'log level', setLogLevel)
  .argument('<manifest>', 'The trait manifest file')
  .argument('<search_dir>', 'The directory to search for trait images')
  .action(async (manifest, search_dir, opts, cmd) => {
    const { keypair, env, rpc } = cmd.opts();

    const walletKeyPair = loadKeypair(keypair);
    let connection;
    if (rpc !== "") {
      connection = new Connection(rpc);
    }
    else {
      connection = new Connection(clusterApiUrl(env));
    }

    console.log(getTraitManifestCache(manifest));
    let files = await getAllFiles(search_dir);
    let traitManifest: TraitManifest = getTraitManifestCache(manifest);
    addFilesToTraitManifest(files, traitManifest);
  });

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function setLogLevel(value, prev) {
  if (value === undefined || value === null) {
    return;
  }
  log.info("setting the log value to: " + value);
  log.setLevel(value);
}

function loadKeypair(keypairPath) {
  const decodedKey = new Uint8Array(
    JSON.parse(
      fs.readFileSync(keypairPath).toString()
    ));

  return Keypair.fromSecretKey(decodedKey);
}

program
  .version("0.0.1")
  .description("CLI for migrating collections to Fusion.")
  .parse(process.argv);