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
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import * as bs58 from 'bs58';
async function getElgamalKeypair(
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

  return new Uint8Array([]);
}

import { Button } from 'antd';
export const Demo = () => {
  const mint = new PublicKey('D26Pw8hk4eyXCsZWVskA51YF7ntf6LAV2JdhgdSeVy6L');
  const wallet = useWallet();
  console.log('Demo', wallet);
  return (
    <div className="app">
      <AppBar />
      <Button
        onClick={() => {
          console.log(getElgamalKeypair(wallet, mint));
        }}
      >
        Decrypt
      </Button>
    </div>
  );
}

export const App = () => {
  console.log('App');
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
