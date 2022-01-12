import * as React from "react";
import { BrowserRouter, Switch, Route } from 'react-router-dom';
import { hot } from "react-hot-loader";

import { CoingeckoProvider } from '../contexts/coingecko';
import { ConnectionProvider } from '../contexts/ConnectionContext';
import { LoaderProvider } from '../components/Loader';
import { SPLTokenListProvider } from '../contexts/tokenList';
import { WalletProvider } from '../contexts/WalletContext';
import { WasmProvider } from '../contexts/WasmContext';
import { AppLayout } from './Layout';

import { GalleryView } from '../views/GalleryView';
import { StealthView } from '../views/StealthView';

export const App = () => {
  return (
    <BrowserRouter>
      <WasmProvider>
      <ConnectionProvider>
      <WalletProvider>
      <SPLTokenListProvider>
      <CoingeckoProvider>
      <LoaderProvider>
        <AppLayout>
          <Switch>
            <Route exact path="/" component={GalleryView} />
            <Route exact path="/stealth" component={StealthView} />
          </Switch>
        </AppLayout>
      </LoaderProvider>
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
