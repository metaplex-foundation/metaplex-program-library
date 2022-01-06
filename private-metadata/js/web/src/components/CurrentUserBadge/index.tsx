import React, { useCallback, useState, useEffect, useRef } from 'react';
import { Link } from 'react-router-dom';

import { useWallet } from '@solana/wallet-adapter-react';
import {
  AccountInfo,
  LAMPORTS_PER_SOL,
  PublicKey,
} from '@solana/web3.js';
import { Button, Popover, Select, Tooltip, Modal } from 'antd';
import { WRAPPED_SOL_MINT } from '@project-serum/serum/lib/token-instructions';
import Jazzicon from 'jazzicon';
import { CopyOutlined } from '@ant-design/icons';
import bs58 from 'bs58';

import cogSvg from './cog.svg';
import solSvg from './sol.svg';
import ftxpayPng from './ftxpay.png';
import {
  ENDPOINTS,
  useConnection,
  useConnectionConfig,
} from '../../contexts/ConnectionContext';
import { useWalletModal } from '../../contexts/WalletContext';
import { useSolPrice } from '../../contexts/coingecko';
import { useTokenList } from '../../contexts/tokenList';
import { useNativeAccount } from '../../contexts/accounts';
import { MetaplexModal } from "../../components/MetaplexModal";
import {
  formatNumber,
  formatUSD,
  shortenAddress,
} from '../../utils/common';
import { useQuerySearch } from '../../hooks/useQuerySearch';

export const Identicon = (props: {
  address?: string | PublicKey;
  style?: React.CSSProperties;
  className?: string;
  alt?: string;
}) => {
  const { style, className, alt } = props;
  const address =
    typeof props.address === 'string'
      ? props.address
      : props.address?.toBase58();
  const ref = useRef<HTMLDivElement>();

  useEffect(() => {
    if (address && ref.current) {
      try {
        ref.current.innerHTML = '';
        ref.current.className = className || '';
        ref.current.appendChild(
          Jazzicon(
            style?.width || 16,
            parseInt(bs58.decode(address).toString('hex').slice(5, 15), 16),
          ),
        );
      } catch (err) {
        // TODO
      }
    }
  }, [address, style, className]);

  return (
    <div
      className="identicon-wrapper"
      title={alt}
      ref={ref as any}
      style={props.style}
    />
  );
};

export const Settings = ({
  additionalSettings,
}: {
  additionalSettings?: JSX.Element;
}) => {
  const { connected, disconnect, publicKey } = useWallet();
  const { setVisible } = useWalletModal();
  const open = React.useCallback(() => setVisible(true), [setVisible]);

  return (
    <>
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          paddingTop: '15px',
        }}
      >
        <Identicon
          address={publicKey?.toBase58()}
          style={{
            width: 48,
          }}
        />
        {publicKey && (
          <>
            <Tooltip title="Copy address">
              <div
                style={{
                  fontWeight: 600,
                  letterSpacing: '-0.02em',
                  color: '#FFFFFF',
                }}
                onClick={() =>
                  navigator.clipboard.writeText(publicKey?.toBase58() || '')
                }
              >
                <CopyOutlined />
                &nbsp;{shortenAddress(publicKey?.toBase58())}
              </div>
            </Tooltip>
          </>
        )}
        <br />
        <span
          style={{
            borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
            width: 'calc(100% + 32px)',
            marginTop: 10,
            marginBottom: 10,
          }}
        ></span>
        {additionalSettings}
      </div>
    </>
  );
};

export const TokenCircle = (props: { iconSize?: number , iconFile?: string, style?:React.CSSProperties}) => {
  const { iconSize = 24 ,iconFile=undefined, style={}} = props;
  const filePath = iconFile? iconFile:"/unknown_token.png"
  return (
    <span
      style={{
        background: 'rgba(255, 255, 255, 0.05)',
        borderRadius: '50%',
        height: iconSize,
        width: iconSize,
        display: 'inline-flex',
        overflow: 'hidden',
        ...style
      }}
    >
      <img src={filePath}/>
    </span>
  );
};

const btnStyle: React.CSSProperties = {
  border: 'none',
  height: 40,
};

const AddFundsModal = (props: {
  showAddFundsModal: any;
  setShowAddFundsModal: any;
  balance: number;
  publicKey: PublicKey;
}) => {
  return (
    <MetaplexModal
      visible={props.showAddFundsModal}
      onCancel={() => props.setShowAddFundsModal(false)}
      title="Add Funds"
      bodyStyle={{
        alignItems: 'start',
      }}
    >
      <div style={{ maxWidth: '100%' }}>
        <p style={{ color: 'white' }}>
          We partner with <b>FTX</b> to make it simple to start purchasing
          digital collectibles.
        </p>
        <div
          style={{
            width: '100%',
            background: '#242424',
            borderRadius: 12,
            marginBottom: 10,
            height: 50,
            display: 'flex',
            alignItems: 'center',
            padding: '0 10px',
            justifyContent: 'space-between',
            fontWeight: 700,
          }}
        >
          <span style={{ color: 'rgba(255, 255, 255, 0.5)' }}>Balance</span>
          <span>
            {formatNumber.format(props.balance)}&nbsp;&nbsp;
            <span
              style={{
                borderRadius: '50%',
                background: 'black',
                display: 'inline-block',
                padding: '1px 4px 4px 4px',
                lineHeight: 1,
              }}
            >
              <img src={solSvg} width="10" />
            </span>{' '}
            SOL
          </span>
        </div>
        <p>
          If you have not used FTX Pay before, it may take a few moments to get
          set up.
        </p>
        <Button
          onClick={() => props.setShowAddFundsModal(false)}
          style={{
            background: '#454545',
            borderRadius: 14,
            width: '30%',
            padding: 10,
            height: 'auto',
          }}
        >
          Close
        </Button>
        <Button
          onClick={() => {
            window.open(
              `https://ftx.com/pay/request?coin=SOL&address=${props.publicKey?.toBase58()}&tag=&wallet=sol&memoIsRequired=false`,
              '_blank',
              'resizable,width=680,height=860',
            );
          }}
          style={{
            background: 'black',
            borderRadius: 14,
            width: '68%',
            marginLeft: '2%',
            padding: 10,
            height: 'auto',
            borderColor: 'black',
          }}
        >
          <div
            style={{
              display: 'flex',
              placeContent: 'center',
              justifyContent: 'center',
              alignContent: 'center',
              alignItems: 'center',
              fontSize: 16,
            }}
          >
            <span style={{ marginRight: 5 }}>Sign with</span>
            <img src={ftxpayPng} width="80" />
          </div>
        </Button>
      </div>
    </MetaplexModal>
  );
};

export const CurrentUserBadge = (props: {
  showBalance?: boolean;
  showAddress?: boolean;
  iconSize?: number;
}) => {
  const { wallet, publicKey, disconnect } = useWallet();
  const { account } = useNativeAccount();
  const solPrice = useSolPrice();
  const [showAddFundsModal, setShowAddFundsModal] = useState<Boolean>(false);

  if (!wallet || !publicKey) {
    return null;
  }
  const balance = (account?.lamports || 0) / LAMPORTS_PER_SOL;
  const balanceInUSD = balance * solPrice;
  const solMintInfo = useTokenList().tokenMap.get(WRAPPED_SOL_MINT.toString());
  const iconStyle: React.CSSProperties = {
    display: 'flex',
    width: props.iconSize,
    borderRadius: 50,
  };

  let name = props.showAddress ? shortenAddress(`${publicKey}`) : '';
  const unknownWallet = wallet as any;
  if (unknownWallet.name && !props.showAddress) {
    name = unknownWallet.name;
  }

  let image = <Identicon address={publicKey?.toBase58()} style={iconStyle} />;

  if (unknownWallet.image) {
    image = <img src={unknownWallet.image} style={iconStyle} />;
  }

  return (
    <div className="wallet-wrapper">
      {props.showBalance && (
        <span>
          {formatNumber.format((account?.lamports || 0) / LAMPORTS_PER_SOL)} SOL
        </span>
      )}

      <Popover
        trigger="click"
        placement="bottomRight"
        content={
          <Settings
            additionalSettings={
              <div
                style={{
                  width: 250,
                }}
              >
                <h5
                  style={{
                    color: 'rgba(255, 255, 255, 0.7)',
                    letterSpacing: '0.02em',
                  }}
                >
                  BALANCE
                </h5>
                <div
                  style={{
                    marginBottom: 10,
                  }}
                >
                  <TokenCircle
                    iconFile={solMintInfo ? solMintInfo.logoURI : ''}
                  />
                  &nbsp;
                  <span
                    style={{
                      fontWeight: 600,
                      color: '#FFFFFF',
                    }}
                  >
                    {formatNumber.format(balance)} SOL
                  </span>
                  &nbsp;
                  <span
                    style={{
                      color: 'rgba(255, 255, 255, 0.5)',
                    }}
                  >
                    {formatUSD.format(balanceInUSD)}
                  </span>
                  &nbsp;
                </div>
                <div
                  style={{
                    display: 'flex',
                    marginBottom: 10,
                  }}
                >
                  <Button
                    className="metaplex-button-default"
                    onClick={() => setShowAddFundsModal(true)}
                    style={btnStyle}
                  >
                    Add Funds
                  </Button>
                  &nbsp;&nbsp;
                  <Button
                    className="metaplex-button-default"
                    onClick={disconnect}
                    style={btnStyle}
                  >
                    Disconnect
                  </Button>
                </div>
              </div>
            }
          />
        }
      >
        <Button className="wallet-key">
          {image}
          {name && (
            <span
              style={{
                marginLeft: '0.5rem',
                fontWeight: 600,
              }}
            >
              {name}
            </span>
          )}
        </Button>
      </Popover>
      <AddFundsModal
        setShowAddFundsModal={setShowAddFundsModal}
        showAddFundsModal={showAddFundsModal}
        publicKey={publicKey}
        balance={balance}
      />
    </div>
  );
};

export const Cog = () => {
  const { endpoint } = useConnectionConfig();
  const routerSearchParams = useQuerySearch();
  const { setVisible } = useWalletModal();
  const open = useCallback(() => setVisible(true), [setVisible]);

  return (
    <div className="wallet-wrapper">
      <Popover
        trigger="click"
        placement="bottomRight"
        content={
          <div
            style={{
              width: 250,
            }}
          >
            <h5
              style={{
                color: 'rgba(255, 255, 255, 0.7)',
                letterSpacing: '0.02em',
              }}
            >
              NETWORK
            </h5>
            <Select
              onSelect={network => {
                // Reload the page, forward user selection to the URL querystring.
                // The app will be re-initialized with the correct network
                // (which will also be saved to local storage for future visits)
                // for all its lifecycle.

                // Because we use react-router's HashRouter, we must append
                // the query parameters to the window location's hash & reload
                // explicitly. We cannot update the window location's search
                // property the standard way, see examples below.

                // doesn't work: https://localhost/?network=devnet#/
                // works: https://localhost/#/?network=devnet
                const windowHash = window.location.hash;
                routerSearchParams.set('network', network as any);
                const nextLocationHash = `${
                  windowHash.split('?')[0]
                }?${routerSearchParams.toString()}`;
                window.location.hash = nextLocationHash;
                window.location.reload();
              }}
              value={endpoint.name}
              bordered={false}
              style={{
                background: 'rgba(255, 255, 255, 0.05)',
                borderRadius: 8,
                width: '100%',
                marginBottom: 10,
              }}
            >
              {ENDPOINTS.map(({ name }) => (
                <Select.Option value={name} key={name}>
                  {name}
                </Select.Option>
              ))}
            </Select>

            <Button
              className="metaplex-button-default"
              style={btnStyle}
              onClick={open}
            >
              Change wallet
            </Button>
          </div>
        }
      >
        <Button className="wallet-key">
          <img src={cogSvg} />
        </Button>
      </Popover>
    </div>
  );
};

export const CurrentUserBadgeMobile = (props: {
  showBalance?: boolean;
  showAddress?: boolean;
  iconSize?: number;
  closeModal?: any;
}) => {
  const { wallet, publicKey, disconnect } = useWallet();
  const { account } = useNativeAccount();
  const solPrice = useSolPrice();

  const [showAddFundsModal, setShowAddFundsModal] = useState<Boolean>(false);

  if (!wallet || !publicKey) {
    return null;
  }
  const balance = (account?.lamports || 0) / LAMPORTS_PER_SOL;
  const balanceInUSD = balance * solPrice;

  const iconStyle: React.CSSProperties = {
    display: 'flex',
    width: props.iconSize,
    borderRadius: 50,
  };

  let name = props.showAddress ? shortenAddress(`${publicKey}`) : '';
  const unknownWallet = wallet as any;
  if (unknownWallet.name && !props.showAddress) {
    name = unknownWallet.name;
  }

  let image = <Identicon address={publicKey?.toBase58()} style={iconStyle} />;

  if (unknownWallet.image) {
    image = <img src={unknownWallet.image} style={iconStyle} />;
  }

  return (
    <div className="current-user-mobile-badge">
      <div className="mobile-badge">
        {image}
        {name && (
          <span
            style={{
              marginLeft: '0.5rem',
              fontWeight: 600,
            }}
          >
            {name}
          </span>
        )}
      </div>
      <div className="balance-container">
        <span className="balance-title">Balance</span>
        <span>
          <span className="sol-img-wrapper">
            <img src={solSvg} width="10" />
          </span>{' '}
          {formatNumber.format(balance)}&nbsp;&nbsp; SOL{' '}
          <span
            style={{
              marginLeft: 5,
              fontWeight: 'normal',
              color: 'rgba(255, 255, 255, 0.5)',
            }}
          >
            {formatUSD.format(balanceInUSD)}
          </span>
        </span>
      </div>
      <div className="actions-buttons">
        <Button
          className="secondary-btn"
          onClick={() => {
            props.closeModal ? props.closeModal() : null;
            setShowAddFundsModal(true);
          }}
        >
          Add Funds
        </Button>
        &nbsp;&nbsp;
        <Button className="black-btn" onClick={disconnect}>
          Disconnect
        </Button>
      </div>
      <AddFundsModal
        setShowAddFundsModal={setShowAddFundsModal}
        showAddFundsModal={showAddFundsModal}
        publicKey={publicKey}
        balance={balance}
      />
    </div>
  );
};
