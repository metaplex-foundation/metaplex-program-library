import os from 'os';
import yaml from 'yaml';
import { keypairIdentity, Metaplex } from '@metaplex-foundation/js';
import * as web3 from '@solana/web3.js';
import * as fs from 'fs';

export async function use_metaplex(keypair: string, env: web3.Cluster, rpc: string) {
  const solanaConfig = loadSolanaConfigFile();
  let connection;

  const selectedRPC = rpc || solanaConfig.json_rpc_url;
  const selectedKeypairPath = keypair || solanaConfig.keypair_path;

  if (selectedRPC) {
    connection = new web3.Connection(selectedRPC, {
      confirmTransactionInitialTimeout: 360000,
    });
  } else {
    connection = new web3.Connection(web3.clusterApiUrl(env), {
      confirmTransactionInitialTimeout: 360000,
    });
  }

  // Load a local keypair.
  const keypairFile = fs.readFileSync(selectedKeypairPath);
  const wallet = web3.Keypair.fromSecretKey(Buffer.from(JSON.parse(keypairFile.toString())));

  const metaplex = new Metaplex(connection);
  // Use it in the SDK.
  metaplex.use(keypairIdentity(wallet));

  return metaplex;
}

export const loadSolanaConfigFile = () => {
  try {
    const path = os.homedir() + '/.config/solana/cli/config.yml';
    const solanaConfigFile = fs.readFileSync(path);
    const config = yaml.parse(solanaConfigFile.toString());
    return config;
  } catch (e) {
    return {};
  }
};

export enum EscrowAuthority {
  TokenOwner = 0,
  Creator = 1,
}

// Creating a replacer to properly JSON stringify Maps.
export function map_replacer(key, value) {
  if (value instanceof Map) {
    return {
      dataType: 'Map',
      value: Array.from(value.entries()), // or with spread: value: [...value]
    };
  } else if (value instanceof Set) {
    return {
      dataType: 'Set',
      value: Array.from(value.values()),
    };
  } else {
    return value;
  }
}
