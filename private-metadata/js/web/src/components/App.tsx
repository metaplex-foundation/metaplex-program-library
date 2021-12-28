import * as React from "react";
import { BrowserRouter, Switch, Route } from 'react-router-dom';
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

export const Demo = () => {
  return (
    <div className="app">
      <CurrentUserBadge
        showBalance={false}
        showAddress={true}
        iconSize={24}
      />
      <Cog />
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
