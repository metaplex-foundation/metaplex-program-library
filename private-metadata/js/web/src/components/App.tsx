import * as React from "react";
import { BrowserRouter, Switch, Route, Link } from 'react-router-dom';
import { hot } from "react-hot-loader";

import { CoingeckoProvider } from '../contexts/coingecko';
import { ConnectionProvider } from '../contexts/ConnectionContext';
import { SPLTokenListProvider } from '../contexts/tokenList';
import { WalletProvider } from '../contexts/WalletContext';
import {
  Cog,
  CurrentUserBadge,
} from './CurrentUserBadge';

import { shortenAddress } from '../utils/common';
import { Tooltip } from 'antd';
import { CopyOutlined } from '@ant-design/icons';


import { ConnectButton } from './ConnectButton';
import { useWallet } from '@solana/wallet-adapter-react';
export const LogoLink = () => {
  return (
    <Link to={`/`}>
      <p className={"janus-logo"}>Janus</p>
    </Link>
  );
};

export const AppBar = () => {
  const { connected } = useWallet();
  return (
    <>
      <div id="desktop-navbar">
        <div className="app-left">
          <LogoLink />
        </div>
        <div className="app-right">
          {/*!connected && (
            <HowToBuyModal buttonClassName="modal-button-default" />
          )*/}
          {!connected && (
            <ConnectButton style={{ height: 48 }} allowWalletChange />
          )}
          {connected && (
            <>
              <CurrentUserBadge
                showBalance={false}
                showAddress={true}
                iconSize={24}
              />
              <Cog />
            </>
          )}
        </div>
      </div>
    </>
  );
};

import { WalletSigner } from "../contexts/WalletContext";
import { Connection, PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import * as bs58 from 'bs58';
import init, {
  elgamal_keypair_from_signature,
  elgamal_decrypt_u32,
} from '../utils/privateMetadata/private_metadata_js';
import { decodePrivateMetadata } from '../utils/privateSchema';
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
): Promise<Uint8Array> {
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

  await init();
  const elgamalKeypair = elgamal_keypair_from_signature([...signature]);

  const privateMetadataKey = await getPrivateMetadata(address);
  const privateMetadataAccount = await connection.getAccountInfo(privateMetadataKey);
  const privateMetadata = decodePrivateMetadata(privateMetadataAccount.data);

  console.log(privateMetadata);

  const input = Buffer.from(await (await fetch(privateMetadata.uri)).arrayBuffer());
  const iv = input.slice(0, 16);

  console.log('Initialization vector', iv);

  const key = Buffer.concat(privateMetadata.encryptedCipherKey.map(
    chunk => (
      Buffer.from(elgamal_decrypt_u32(
        elgamalKeypair,
        { bytes: [...chunk] },
      ))
    )));

  console.log(`Decoded cipher key bytes: ${[...key]}`);
  console.log(`Decoded cipher key: ${bs58.encode(key)}`);

  return new Uint8Array([]);
}

import { Button } from 'antd';
import { useConnection } from '../contexts/ConnectionContext';
export const Demo = () => {
  const mint = new PublicKey('D26Pw8hk4eyXCsZWVskA51YF7ntf6LAV2JdhgdSeVy6L');
  const connection = useConnection();
  const wallet = useWallet();
  console.log('Demo', wallet, connection);
  return (
    <div className="app">
      <AppBar />
      <Button
        onClick={() => {
          console.log(getElgamalKeypair(connection, wallet, mint));
        }}
      >
        Decrypt
      </Button>
    </div>
  );
}

export const App = () => {
  return (
    <ConnectionProvider>
      <WalletProvider>
        <SPLTokenListProvider>
        <CoingeckoProvider>
          <BrowserRouter>
            <Switch>
              <Route path="/" component={() => (
                <Demo />
              )} />
            </Switch>
          </BrowserRouter>
        </CoingeckoProvider>
        </SPLTokenListProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}

declare let module: Record<string, unknown>;

export default hot(module)(App);
