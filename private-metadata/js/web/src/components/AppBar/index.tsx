import React from "react";
import { Link } from 'react-router-dom';

import { useWallet } from '@solana/wallet-adapter-react';

import {
  Cog,
  CurrentUserBadge,
} from '../CurrentUserBadge';
import { ConnectButton } from '../ConnectButton';

export const LogoLink = () => {
  return (
    <Link to={`/`}>
      <p className={"janus-logo"}>JANUS</p>
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

