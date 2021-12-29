import * as React from "react";
import { BrowserRouter, Switch, Route } from 'react-router-dom';
import { hot } from "react-hot-loader";

import { CoingeckoProvider } from '../contexts/coingecko';
import { ConnectionProvider } from '../contexts/ConnectionContext';
import { SPLTokenListProvider } from '../contexts/tokenList';
import { WalletProvider } from '../contexts/WalletContext';
import { WasmProvider } from '../contexts/WasmContext';
import { AppLayout } from './Layout';

import { shortenAddress } from '../utils/common';
import { Layout, Tooltip } from 'antd';
import { CopyOutlined } from '@ant-design/icons';

const { Header, Content } = Layout;

import { WalletSigner } from "../contexts/WalletContext";
import { useWasmConfig, WasmConfig } from "../contexts/WasmContext";
import { wrap } from 'comlink';
import { Connection, PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import * as bs58 from 'bs58';
import { decodePrivateMetadata, PrivateMetadataAccount } from '../utils/privateSchema';
import { PRIVATE_METADATA_PROGRAM_ID } from '../utils/ids';
async function getPrivateMetadata(
  mint: PublicKey,
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        mint.toBuffer(),
      ],
      PRIVATE_METADATA_PROGRAM_ID,
    )
  )[0];
};

async function getElgamalKeypair(
  connection: Connection,
  wallet: WalletSigner,
  address: PublicKey,
  wasm: WasmConfig,
): Promise<any> { // TODO: type
  let transaction = new Transaction();
  transaction.add(new TransactionInstruction({
    programId: address, // mint
    keys: [],
    data: Buffer.from("ElGamalSecretKey"),
  }));

  const blockhash_bytes = 32;
  transaction.recentBlockhash = bs58.encode(
    new Array(blockhash_bytes).fill(0)
  );

  transaction.setSigners(wallet.publicKey);

  const signature = await wallet.signMessage(
      transaction.compileMessage().serialize());
  if (signature === null) {
    throw new Error(`Failed ElGamal keypair generation: signature`);
  }
  console.log('Signature {}', bs58.encode(signature));

  return wasm.elgamalKeypairFromSignature([...signature]);
}

async function getCipherKey(
  connection: Connection,
  wallet: WalletSigner,
  address: PublicKey,
  privateMetadata: PrivateMetadataAccount,
  wasm: WasmConfig,
): Promise<Buffer> {
  const elgamalKeypairRes = await getElgamalKeypair(
    connection, wallet, address, wasm);

  if (elgamalKeypairRes.Err) {
    throw new Error(elgamalKeypairRes.Err);
  }

  const elgamalKeypair = elgamalKeypairRes.Ok;

  return Buffer.concat(await Promise.all(privateMetadata.encryptedCipherKey.map(
    async (chunk) => {
      const decryptWorker = new Worker(new URL(
        '../utils/decryptWorker.js',
        import.meta.url,
      ));
      const decryptWorkerApi = wrap(decryptWorker) as any;
      console.log('Sending chunk to worker', chunk);
      const result: any = await decryptWorkerApi.decrypt(elgamalKeypair, chunk);
      if (result.Err) {
        throw new Error(result.Err);
      }
      return Buffer.from(result.Ok);
    }))
  );
}

import { Button, Input } from 'antd';
import { useWallet } from '@solana/wallet-adapter-react';
import { useConnection } from '../contexts/ConnectionContext';
import { useLocalStorageState } from '../utils/common';
import * as CryptoJS from 'crypto-js';
import { drawArray } from '../utils/image';
import { notify } from '../utils/common';
export const Demo = () => {
  const connection = useConnection();
  const wallet = useWallet();
  const wasmConfig = useWasmConfig();

  const [mint, setMint] = useLocalStorageState('mint', '');
  const [privateMetadata, setPrivateMetadata]
      = React.useState<PrivateMetadataAccount | null>(null);
  const [privateImage, setPrivateImage]
      = React.useState<Buffer | null>(null);
  const [decryptedImage, setDecryptedImage]
      = React.useState<Buffer | null>(null);

  const parseAddress = (address: string): PublicKey | null => {
    try {
      return new PublicKey(address);
    } catch {
      return null;
    }
  };

  React.useEffect(() => {
    const mintKey = parseAddress(mint);
    if (mintKey === null) return;

    const wrap = async () => {
      const privateMetadataKey = await getPrivateMetadata(mintKey);
      const privateMetadataAccount = await connection.getAccountInfo(privateMetadataKey);
      const privateMetadata = decodePrivateMetadata(privateMetadataAccount.data);

      setPrivateMetadata(privateMetadata);
    };
    wrap();
  }, [connection, mint]);

  React.useEffect(() => {
    if (privateMetadata === null) return;
    const wrap = async () => {
      setPrivateImage(Buffer.from(
        await (
          await fetch(privateMetadata.uri)
        ).arrayBuffer()
      ));
    };
    wrap();
  }, [privateMetadata]);

  return (
    <div className="app stack">
      <label className="action-field">
        <span className="field-title">NFT Mint</span>
        <Input
          id="mint-text-field"
          value={mint}
          onChange={(e) => setMint(e.target.value)}
          style={{ fontFamily: 'Monospace' }}
        />
      </label>
      {privateImage && (
        <div>
          <img
            style={{ margin: 'auto', display: 'block'}}
            src={"data:image/bmp;base64," + drawArray(privateImage, 8)}
          />
        </div>
      )}
      {decryptedImage && (
        <div>
          <img
            style={{ margin: 'auto', display: 'block'}}
            src={"data:image/png;base64," + decryptedImage.toString('base64')}
          />
        </div>
      )}
      <Button
        style={{ width: '100%' }}
        className="metaplex-button"
        disabled={!privateMetadata || !wallet?.connected}
        onClick={() => {
          if (!privateMetadata) {
            return;
          }
          const mintKey = parseAddress(mint);
          if (mintKey === null) {
            console.error(`Failed to parse mint ${mint}`);
          }
          const run = async () => {
            const cipherKey = await getCipherKey(
              connection, wallet, mintKey, privateMetadata, wasmConfig);
            console.log(`Decoded cipher key bytes: ${[...cipherKey]}`);
            console.log(`Decoded cipher key: ${bs58.encode(cipherKey)}`);

            const input = Buffer.from(await (await fetch(privateMetadata.uri)).arrayBuffer());
            const AES_BLOCK_SIZE = 16;
            const iv = input.slice(0, AES_BLOCK_SIZE);

            // expects a base64 encoded string by default (openSSL mode?)
            // also possible to give a `format: CryptoJS.format.Hex`
            const ciphertext = input.slice(AES_BLOCK_SIZE).toString('base64');
            // this can be a string but I couldn't figure out which encoding it
            // wants so just make it a WordArray
            const cipherKeyWordArray
              = CryptoJS.enc.Base64.parse(cipherKey.toString('base64'));
            const ivWordArray
              = CryptoJS.enc.Base64.parse(iv.toString('base64'));

            const decrypted = CryptoJS.AES.decrypt(
              ciphertext,
              cipherKeyWordArray,
              { iv: ivWordArray },
            );

            setDecryptedImage(Buffer.from(decrypted.toString(), 'hex'));
          }
          const wrap = async () => {
            try {
              await run();
            } catch (err) {
              // console.error(err);
              notify({
                message: 'Failed to decrypt image',
                description: err.message,
              })
            }
          };
          wrap();
        }}
      >
        Decrypt
      </Button>
    </div>
  );
}

export const App = () => {
  return (
    <BrowserRouter>
      <WasmProvider>
      <ConnectionProvider>
      <WalletProvider>
      <SPLTokenListProvider>
      <CoingeckoProvider>
        <AppLayout>
          <Switch>
            <Route path="/" component={() => (
              <Demo />
            )} />
          </Switch>
        </AppLayout>
      </CoingeckoProvider>
      </SPLTokenListProvider>
      </WalletProvider>
      </ConnectionProvider>
      </WasmProvider>
    </BrowserRouter>
  );
}

declare let module: Record<string, unknown>;

export default hot(module)(App);
