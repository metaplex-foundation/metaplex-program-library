import * as React from "react";
import { Link } from "react-router-dom";
import { RouteComponentProps } from 'react-router-dom';

import { Button, Col, Row } from 'antd';
import { useWallet } from '@solana/wallet-adapter-react';
import {
  PublicKey,
} from '@solana/web3.js';
import { AccountLayout } from '@solana/spl-token';
import * as BN from 'bn.js';

import { useWindowDimensions } from '../../components/AppBar';
import { useLoading } from '../../components/Loader';
import {
  CachedImageContent,
} from '../../components/ArtContent';
import {
  explorerLinkCForAddress,
  useConnection,
} from '../../contexts/ConnectionContext';
import {
  notify,
  useLocalStorageState,
} from '../../utils/common';
import { getMultipleAccounts } from '../../utils/getMultipleAccounts';
import {
  getMetadata,
  getPrivateMetadata,
  TOKEN_PROGRAM_ID,
} from '../../utils/ids';
import {
  decodeMetadata,
  Metadata,
} from '../../utils/publicSchema';

export const GalleryView = (
  props: RouteComponentProps<{}>,
) => {
  // contexts
  const connection = useConnection();
  const wallet = useWallet();
  const { loading, setLoading } = useLoading();

  // async useEffect set
  const [lastFetchedPubkey, setLastFetchedPubkey]
      = useLocalStorageState('lastFetchedPubkey', '');
  const [galleryMints, setGalleryMints]
      = useLocalStorageState('galleryMints', []);
  const [publicManifests, setPublicManifests]
      = useLocalStorageState('publicManifests', []);

  React.useEffect(() => {
    if (!wallet.publicKey) {
      setLastFetchedPubkey('');
      setGalleryMints([]);
      setPublicManifests([]);
      return;
    }

    // janky memo
    if (lastFetchedPubkey === wallet.publicKey.toBase58()) {
      return;
    }

    // seems a bit race-conditioney...
    setLastFetchedPubkey(wallet.publicKey.toBase58());
    setGalleryMints([]);
    setPublicManifests([]);

    const run = async () => {
      const tokenAccounts = await connection.getTokenAccountsByOwner(
        wallet.publicKey,
        { programId: TOKEN_PROGRAM_ID },
      );
      const accountsDecoded = tokenAccounts.value.map(
        v => AccountLayout.decode(v.account.data)
      );

      let mints = accountsDecoded
        .filter(r => new BN(r.amount, 'le').toNumber() > 0)
        .map(r => new PublicKey(r.mint))
        .sort((lft, rht) => lft.toBase58().localeCompare(rht.toBase58()))
      ;

      const privateMetadatas = await Promise.all(mints.map(m => getPrivateMetadata(m)));
      const { array: pmAccounts } = await getMultipleAccounts(
        connection,
        privateMetadatas.map(p => p.toBase58()),
        'singleGossip',
      );

      mints = mints.filter((mint, idx) => !!pmAccounts[idx]?.data);

      const metadatas = await Promise.all(mints.map(m => getMetadata(m)));

      const { keys, array } = await getMultipleAccounts(
        connection,
        metadatas.map(p => p.toBase58()),
        'singleGossip',
      );

      const decoded = array.map(account => decodeMetadata(account.data));

      const responses = await Promise.all(
        decoded.map(
          pm => fetch(pm.data.uri)
        )
      );
      const manifests = await Promise.all(
        responses.map(r => r.json())
      );

      setGalleryMints(mints.map(p => p.toBase58()));
      setPublicManifests(manifests);
    };

    const wrap = async () => {
      try {
        await run();
      } catch (err) {
        console.error(err);
        notify({
          message: 'Failed to fetch NFT gallery',
          description: err.message,
        })
      } finally {
        setLoading(false);
      }
    };

    setLoading(true);
    wrap();
  }, [wallet]);

  const parseAddress = (address: string): PublicKey | null => {
    try {
      return new PublicKey(address);
    } catch {
      return null;
    }
  };

  // TODO: more robust
  const maxWidth = 1440;
  const outerPadding = 96 * 2;
  const columnsGap = 40;
  const maxColumns = 4;
  const columnWidth = (maxWidth - outerPadding - columnsGap * (maxColumns - 1)) / maxColumns;

  const tilePadding = 0;
  const imageWidth = columnWidth - tilePadding * 2;

  const { width } = useWindowDimensions();
  const sizedColumns = (width : number) => {
           if (width > columnWidth * 4 + columnsGap * 3 + outerPadding) {
      return 4;
    } else if (width > columnWidth * 3 + columnsGap * 2 + outerPadding) {
      return 3;
    } else if (width > columnWidth * 2 + columnsGap * 1 + outerPadding) {
      return 2;
    } else {
      return 1;
    }
  };
  const cols = sizedColumns(width);

  if (!wallet.publicKey
      || wallet.publicKey.toBase58() !== lastFetchedPubkey
      || galleryMints.length === 0
      || publicManifests.length === 0) {
    return (
      <div className="app stack" style={{ margin: 'auto' }}>
        <p className={"text-title"}>
          NFT Gallery
        </p>
        <p>
          Connect your wallet to view your NFTs
        </p>
      </div>
    );
  }

  return (
    <div className="app stack" style={{ margin: 'auto' }}>
      <p className={"text-title"}>
        NFT Gallery
      </p>
      {!wallet.connected && (
        <p className={"text-subtitle"}>
          Connect your wallet to view your NFTs
        </p>
      )}
      <div>
        {galleryMints.length > 0
          && publicManifests.length > 0
          && function () {
            const rows = Math.ceil(galleryMints.length / cols);
            const colSections = 24 / cols;
            return (
              <React.Fragment>
              {[...Array(rows).keys()].map(rowIdx => {
              return (
                <Row key={rowIdx}>
                  {[...Array(cols).keys()].map(colIdx => {
                    const mintIdx = rowIdx * cols + colIdx;
                    if (mintIdx >= galleryMints.length)
                      return null;
                    const mint = galleryMints[mintIdx];
                    const manifest = publicManifests[mintIdx];
                    return (
                      <Col key={colIdx} span={colSections}>
                        <div>
                          <CachedImageContent
                            uri={manifest.image}
                            className={"fullAspectRatio"}
                          />
                          <div>
                            <p
                              className={"text-subtitle"}
                              style={{
                                fontSize: '15px',
                              }}
                            >
                              {manifest.name}
                            </p>
                          </div>
                          <div>
                            {explorerLinkCForAddress(mint, connection)}
                          </div>
                          <span>
                          <Button
                            style={{
                              borderRadius: "30px",
                              height: "35px",
                            }}
                          >
                            <Link
                              to={`/stealth?mint=${mint}`}
                              style={{
                                color: 'inherit',
                                display: 'block',
                              }}
                            >
                              View Details
                            </Link>
                          </Button>
                          </span>
                        </div>
                      </Col>
                    );
                  })}
                </Row>
              );
            })}
            </React.Fragment>
          );
          }()
        }
      </div>
    </div>
  );
}
