import * as React from "react";
import { RouteComponentProps } from 'react-router-dom';

import {
  Button,
  Input,
  List,
} from 'antd';
import {
  DeleteOutlined,
} from '@ant-design/icons';
import {
  Connection,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import * as bs58 from 'bs58';

import { CollapsePanel } from '../../components/CollapsePanel';
import {
  useLoading,
  incLoading,
  decLoading,
} from '../../components/Loader';
import {
  getElgamalKeypair,
  useWasmConfig,
  WasmConfig,
} from "../../contexts/WasmContext";
import { WalletSigner } from "../../contexts/WalletContext";
import {
  explorerLinkCForAddress,
  sendTransactionWithRetry,
  useConnection,
} from '../../contexts/ConnectionContext';
import {
  notify,
  useLocalStorageState,
} from '../../utils/common';
import {
  getElgamalPubkeyAddress,
  parseAddress,
  parseKeypair,
  STEALTH_PROGRAM_ID,
} from '../../utils/ids';
import {
  decodeEncryptionKeyBuffer,
  EncryptionKeyBuffer,
} from '../../utils/stealthSchema';
import {
  explorerLinkFor,
} from '../../utils/transactions';

const publish = async (
  connection: Connection,
  wallet: WalletSigner,
  mintKey: PublicKey,
  wasm: WasmConfig,
) => {
  const elgamalPubkeyAddress = await getElgamalPubkeyAddress(
    wallet.publicKey, mintKey);
  if (await connection.getAccountInfo(elgamalPubkeyAddress) !== null) {
    throw new Error(`Encryption key is already published for ${wallet.publicKey.toBase58()}:${mintKey.toBase58()}`);
  }
  const elgamalKeypairRes = await getElgamalKeypair(
    connection, wallet, mintKey, wasm);

  if (elgamalKeypairRes.Err) {
    throw new Error(elgamalKeypairRes.Err);
  }

  const elgamalKeypair = elgamalKeypairRes.Ok;

  const instructions = [
    // InitTransfer
    new TransactionInstruction({
      programId: STEALTH_PROGRAM_ID,
      keys: [
        {
          pubkey: wallet.publicKey,
          isSigner: true,
          isWritable: true,
        },
        {
          pubkey: mintKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: elgamalPubkeyAddress,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: SYSVAR_RENT_PUBKEY,
          isSigner: false,
          isWritable: false,
        },
      ],
      data: Buffer.from([
        5,   // PublishElgamalPubkey...
        ...elgamalKeypair.public,
      ])
    }),
  ];

  const result = await sendTransactionWithRetry(
    connection,
    wallet,
    instructions,
    [],
  );

  console.log(result);
  if (typeof result === "string") {
    throw new Error(result);
  } else {
    notify({
      message: "Publish succeeded",
      description: (
        <a
          href={explorerLinkFor(result.txid, connection)}
          target="_blank"
          rel="noreferrer"
        >
          View transaction on explorer
        </a>
      ),
    });
  }
};

const close = async (
  connection: Connection,
  wallet: WalletSigner,
  mintKey: PublicKey,
) => {
  const instructions = [
    // InitTransfer
    new TransactionInstruction({
      programId: STEALTH_PROGRAM_ID,
      keys: [
        {
          pubkey: wallet.publicKey,
          isSigner: true,
          isWritable: true,
        },
        {
          pubkey: mintKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: await getElgamalPubkeyAddress(wallet.publicKey, mintKey),
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ],
      data: Buffer.from([
        6,   // CloseElgamalPubkey...
      ])
    }),
  ];

  const result = await sendTransactionWithRetry(
    connection,
    wallet,
    instructions,
    [],
  );

  console.log(result);
  if (typeof result === "string") {
    throw new Error(result);
  } else {
    notify({
      message: "Close succeeded",
      description: (
        <a
          href={explorerLinkFor(result.txid, connection)}
          target="_blank"
          rel="noreferrer"
        >
          View transaction on explorer
        </a>
      ),
    });
  }
};

export const PublishView = (
  props: RouteComponentProps<{}>,
) => {
  // contexts
  const connection = useConnection();
  const wallet = useWallet();
  const wasm = useWasmConfig();
  const { loading, setLoading } = useLoading();

  // user inputs
  const [mintStr, setMintStr]
    = useLocalStorageState('publishMintStr', '');

  // async useEffect set
  const [publishedKeys, setPublishedKeys]
    = React.useState<Array<EncryptionKeyBuffer>>([]);

  React.useEffect(() => {
    if (!wallet.publicKey) return;

    const match = bs58.encode([
      3,  // struct key
      ...wallet.publicKey.toBuffer(),  // owner
    ]);

    const run = async () => {
      const keyAccounts = await connection.getProgramAccounts(
        STEALTH_PROGRAM_ID,
        {
          filters: [
            {
              memcmp: {
                offset: 0,
                bytes: match,
              },
            },
          ],
        },
      );

      console.log(keyAccounts);

      const keys = keyAccounts.map(o => {
        return decodeEncryptionKeyBuffer(o.account.data);
      });

      setPublishedKeys(keys);
    };

    const wrap = async () => {
      try {
        await run();
      } catch (err) {
        console.error(err);
        notify({
          message: 'Failed to fetch encryption keys',
          description: err.message,
        })
      } finally {
        setLoading(decLoading);
      }
    };

    setLoading(incLoading);
    wrap();
  }, [connection, wallet]);

  return (
    <div className="app stack" style={{ margin: 'auto' }}>
      <label className="action-field">
        <span className="field-title">Mint</span>
        <Input
          id="compute-buffer-field"
          value={mintStr}
          onChange={(e) => setMintStr(e.target.value)}
          style={{ fontFamily: 'Monospace' }}
        />
      </label>
      <Button
        style={{ width: '100%' }}
        className="metaplex-button"
        disabled={!!loading || !wallet.connected || !parseAddress(mintStr)}
        onClick={() => {
          const mintKey = parseAddress(mintStr);
          if (mintKey === null) {
            console.error(`Failed to parse mint ${mintStr}`);
            return;
          }

          const wrap = async () => {
            try {
              await publish(connection, wallet, mintKey, wasm);
            } catch (err) {
              console.error(err);
              notify({
                message: 'Failed to publish encryption key',
                description: err.message,
              })
            } finally {
              setLoading(decLoading);
            }
          };

          setLoading(incLoading);
          wrap();
        }}
      >
        Publish
      </Button>
      <CollapsePanel
        id="published-encryption-collapse"
        panelName="Published encryption keys"
      >
        <List
          itemLayout="horizontal"
          dataSource={publishedKeys}
          renderItem={key => (
            <List.Item>
              <List.Item.Meta
                avatar={
                  <Button
                    onClick={() => {
                      const wrap = async () => {
                        try {
                          await close(connection, wallet, new PublicKey(key.mint));
                        } catch (err) {
                          console.error(err);
                          notify({
                            message: 'Failed to close encryption key',
                            description: err.message,
                          })
                        } finally {
                          setLoading(decLoading);
                        }
                      };

                      setLoading(incLoading);
                      wrap();
                    }}
                  >
                    <DeleteOutlined />
                  </Button>
                }
                title={(
                  <div>
                    <span className="field-title">Mint{"\u00A0"}</span>
                    {explorerLinkCForAddress(key.mint, connection, false)}
                  </div>
                )}
                description={(
                  <div>
                    <span className="field-title">Encryption key{"\u00A0"}</span>
                    {Buffer.from([...key.elgamalPk]).toString('base64')}
                  </div>
                )}
              />
            </List.Item>
          )}
        />
      </CollapsePanel>
    </div>
  );
}
