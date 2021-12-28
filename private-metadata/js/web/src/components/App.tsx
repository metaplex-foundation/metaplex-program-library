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

export const Demo = () => {
  return (
    <div className="app">
      <AppBar />
      <h1>Hello World!</h1>
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
