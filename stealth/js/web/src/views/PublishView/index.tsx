import * as React from "react";
import { RouteComponentProps } from 'react-router-dom';

import {
  Button,
  Input,
} from 'antd';
import {
  Connection,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';

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
              await close(connection, wallet, mintKey);
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
        Close
      </Button>
    </div>
  );
}
