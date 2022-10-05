#!/usr/bin/env node

import chalk from 'chalk';
import clear from 'clear';
import * as figlet from 'figlet';
import * as path from 'path';
import { program } from 'commander';
import log from 'loglevel';
import * as sdk from '@metaplex-foundation/mpl-token-metadata/src/generated';
import * as web3 from '@solana/web3.js';
import * as fs from 'fs';
import { Keypair } from '@solana/web3.js';
import {
  keypairIdentity,
  Metaplex,
  Nft,
  NftWithToken,
  Sft,
  SftWithToken,
} from '@metaplex-foundation/js';
import { use_metaplex } from './helpers/utils';
import { createConstraintModel, createTrifle, showModel, showTrifle } from './helpers/trifle';
import { findEscrowConstraintModelPda, findTriflePda } from './helpers/pdas';
import { Key } from '@metaplex-foundation/mpl-token-metadata';

clear();
console.log(chalk.green(figlet.textSync('Trifle CLI', { horizontalLayout: 'full' })));

const create = program.command('create');

create
  .command('model')
  .option(
    '-e, --env <string>',
    'Solana cluster env name',
    'devnet', //mainnet-beta, testnet, devnet
  )
  .option('-r, --rpc <string>', 'The endpoint to connect to.')
  .option('-k, --keypair <path>', `Solana wallet location`, '--keypair not provided')
  .option('-l, --log-level <string>', 'log level', setLogLevel)
  .option('-n, --name <string>', 'The name of the constraint model.')
  .option('-s, --schema <string>', 'The schema of the constraint model.')
  .action(async (directory, cmd) => {
    console.log(cmd.opts());
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

    console.log('Constraint Model Created!');
    showModel(metaplex.connection, modelAddr);
  });

create
  .command('trifle')
  .option(
    '-e, --env <string>',
    'Solana cluster env name',
    'devnet', //mainnet-beta, testnet, devnet
  )
  .option('-r, --rpc <string>', 'The endpoint to connect to.')
  .option('-k, --keypair <path>', `Solana wallet location`, '--keypair not provided')
  .option('-l, --log-level <string>', 'log level', setLogLevel)
  .option('-m, --mint <string>', 'The mint of the NFT you want to create a trifle for.')
  .option('-c, --create', 'Create a new base NFT with the Trifle.')
  .option('-u, --uri <string>', 'The URI if creating a new NFT.')
  .option('-n, --name <string>', 'The name if creating a new NFT.')
  .option(
    '-mn, --model-name <string>',
    'The name of the constraint model (Assumes keypair is the same as the Model Authority).',
  )
  .action(async (directory, cmd) => {
    console.log(cmd.opts());
    const { keypair, env, rpc, mint, create, uri, name, modelName } = cmd.opts();

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

    console.log('Trifle Created!');
    showTrifle(metaplex.connection, trifleAddr);
  });

const show = program.command('show');

show
  .command('model')
  .option(
    '-e, --env <string>',
    'Solana cluster env name',
    'devnet', //mainnet-beta, testnet, devnet
  )
  .option('-r, --rpc <string>', 'The endpoint to connect to.')
  .option('-k, --keypair <path>', `Solana wallet location`, '--keypair not provided')
  .option('-l, --log-level <string>', 'log level', setLogLevel)
  .option('-n, --name <string>', 'The name if creating a new NFT.')
  .action(async (directory, cmd) => {
    console.log(cmd.opts());
    const { keypair, env, rpc, name } = cmd.opts();

    const metaplex = await use_metaplex(keypair, env, rpc);

    const modelAddr = await findEscrowConstraintModelPda(metaplex.identity().publicKey, name);
    showModel(metaplex.connection, modelAddr[0]);
  });

show
  .command('trifle')
  .option(
    '-e, --env <string>',
    'Solana cluster env name',
    'devnet', //mainnet-beta, testnet, devnet
  )
  .option('-r, --rpc <string>', 'The endpoint to connect to.')
  .option('-k, --keypair <path>', `Solana wallet location`, '--keypair not provided')
  .option('-l, --log-level <string>', 'log level', setLogLevel)
  .option('-m, --mint <string>', 'The mint of the NFT you want to view the Trifle for.')
  .option('-mn, --model-name <string>', 'The Model the Trifle uses.')
  .option(
    '-mn, --model-name <string>',
    'The name of the constraint model (Assumes keypair is the same as the Model Authority).',
  )
  .action(async (directory, cmd) => {
    console.log(cmd.opts());
    const { keypair, env, rpc, mint, modelName } = cmd.opts();

    const metaplex = await use_metaplex(keypair, env, rpc);

    const modelAddr = await findEscrowConstraintModelPda(metaplex.identity().publicKey, modelName);
    const trifleAddr = await findTriflePda(
      new web3.PublicKey(mint),
      metaplex.identity().publicKey,
      modelAddr[0],
    );
    showTrifle(metaplex.connection, trifleAddr[0]);
  });

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function setLogLevel(value, prev) {
  if (value === undefined || value === null) {
    return;
  }
  log.info('setting the log value to: ' + value);
  log.setLevel(value);
}

program
  .version('0.0.1')
  .description('CLI for controlling and managing Trifle accounts.')
  .parse(process.argv);
