import * as React from "react";
import { Link } from "react-router-dom";

import { Button, Col, Row } from 'antd';
import { useWallet } from '@solana/wallet-adapter-react';
import {
  PublicKey,
} from '@solana/web3.js';
import { AccountLayout } from '@solana/spl-token';
import * as BN from 'bn.js';

import { useWindowDimensions } from '../../components/AppBar';
import {
  useLoading,
  incLoading,
  decLoading,
} from '../../components/Loader';
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
  getStealth,
  TOKEN_PROGRAM_ID,
} from '../../utils/ids';
import {
  decodeMetadata,
} from '../../utils/publicSchema';

export const GalleryView = (
) => {
  // contexts
  const connection = useConnection();
  const wallet = useWallet();
  const { setLoading } = useLoading();

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

      const privateMetadatas = await Promise.all(mints.map(m => getStealth(m)));
      const { array: pmAccounts } = await getMultipleAccounts(
        connection,
        privateMetadatas.map(p => p.toBase58()),
        'singleGossip',
      );

      mints = mints.filter((_, idx) => !!pmAccounts[idx]?.data);

      const metadatas = await Promise.all(mints.map(m => getMetadata(m)));

      const { array } = await getMultipleAccounts(
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
        setLoading(decLoading);
      }
    };

    setLoading(incLoading);
    wrap();
  }, [wallet]);

  // TODO: more robust
  const maxWidth = 1440;
  const outerPadding = 96 * 2;
  const columnsGap = 40;
  const maxColumns = 4;
  const columnWidth = (maxWidth - outerPadding - columnsGap * (maxColumns - 1)) / maxColumns;

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
      || wallet.publicKey.toBase58() !== lastFetchedPubkey) {
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
                              to={`/stealth/view?mint=${mint}`}
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
