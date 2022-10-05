import { keypairIdentity, Metaplex } from '@metaplex-foundation/js';
import * as web3 from '@solana/web3.js';
import * as fs from 'fs';

export async function use_metaplex(keypair: string, env: web3.Cluster, rpc: string) {
  let connection;
  if (rpc !== undefined) {
    console.log(rpc);
    connection = new web3.Connection(rpc, { confirmTransactionInitialTimeout: 360000 });
  } else {
    connection = new web3.Connection(web3.clusterApiUrl(env), {
      confirmTransactionInitialTimeout: 360000,
    });
  }

  // Load a local keypair.
  const keypairFile = fs.readFileSync(keypair);
  const wallet = web3.Keypair.fromSecretKey(Buffer.from(JSON.parse(keypairFile.toString())));

  const metaplex = new Metaplex(connection);
  // Use it in the SDK.
  metaplex.use(keypairIdentity(wallet));

  return metaplex;
}
