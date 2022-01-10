import * as React from "react";
import { RouteComponentProps } from 'react-router-dom';
import queryString from 'query-string';

import {
  Button,
  Card,
  Input,
  Modal,
  Spin,
  Steps,
} from 'antd';
import {
  LoadingOutlined,
} from '@ant-design/icons';
import { useWallet } from '@solana/wallet-adapter-react';
import { wrap } from 'comlink';
import {
  Blockhash,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  TransactionSignature,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import { Token } from '@solana/spl-token';
import * as CryptoJS from 'crypto-js';
import * as bs58 from 'bs58';
import * as BN from 'bn.js';

import { CollapsePanel } from '../../components/CollapsePanel';
import { useLoading } from '../../components/Loader';
import { useWindowDimensions } from '../../components/AppBar';
import {
  CachedImageContent,
  DataUrlImageContent,
} from '../../components/ArtContent';
import {
  explorerLinkCForAddress,
  sendTransactionWithRetry,
  useConnection,
} from '../../contexts/ConnectionContext';
import { WalletSigner } from "../../contexts/WalletContext";
import {
  useWasmConfig,
  WasmConfig,
} from "../../contexts/WasmContext";
import {
  notify,
  shortenAddress,
  useLocalStorageState,
} from '../../utils/common';
import {
  explorerLinkFor,
  sendSignedTransaction,
} from '../../utils/transactions';
import {
  decodePrivateMetadata,
  decodeTransferBuffer,
  PrivateMetadataAccount,
} from '../../utils/privateSchema';
import {
  decodeMetadata,
  Metadata,
} from '../../utils/publicSchema';
import {
  getElgamalPubkeyAddress,
  getPrivateMetadata,
  getMetadata,
  CURVE_DALEK_ONCHAIN_PROGRAM_ID,
  PRIVATE_METADATA_PROGRAM_ID,
  SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TOKEN_METADATA_PROGRAM_ID,
} from '../../utils/ids';

async function getElgamalKeypair(
  connection: Connection,
  wallet: WalletSigner,
  address: PublicKey,
  wasm: WasmConfig,
): Promise<any> { // TODO: type
  const message = `ElGamalSecretKey:${wallet.publicKey.toBase58()}:${address.toBase58()}`;

  // NB / TODO: phantom wallet auto-approve seems to generate a different
  // signature than the normal signMessage...
  const signature = await wallet.signMessage(Buffer.from(message));
  if (signature === null) {
    throw new Error(`Failed ElGamal keypair generation: signature`);
  }
  console.log('Signature', bs58.encode(signature));

  return wasm.elgamalKeypairFromSignature([...signature]);
}

async function getCipherKey(
  connection: Connection,
  wallet: WalletSigner,
  address: PublicKey,
  privateMetadata: PrivateMetadataAccount,
  wasm: WasmConfig,
): Promise<[Buffer, Buffer]> {
  const elgamalKeypairRes = await getElgamalKeypair(
    connection, wallet, address, wasm);

  if (elgamalKeypairRes.Err) {
    throw new Error(elgamalKeypairRes.Err);
  }

  const elgamalKeypair = elgamalKeypairRes.Ok;

  const doDecrypt =
    async (chunk: Uint8Array) => {
      const decryptWorker = new Worker(new URL(
        '../../utils/decryptWorker.js',
        import.meta.url,
      ));
      const decryptWorkerApi = wrap(decryptWorker) as any;
      console.log('Sending chunk to worker', chunk);
      const result: any = await decryptWorkerApi.decrypt(elgamalKeypair, chunk);
      if (result.Err) {
        console.error('Failed decrypt', result.Err, chunk);
        throw new Error(result.Err);
      }
      return Buffer.from(result.Ok);
    };
  return [await doDecrypt(privateMetadata.encryptedCipherKey), elgamalKeypair];
}

const ensureBuffersClosed = async (
  connection: Connection,
  walletKey: PublicKey,
  buffers: Array<PublicKey>,
  checkExists: boolean = true,
) => {
  const infos = checkExists ? await connection.getMultipleAccountsInfo(buffers) : null;
  const ixs: Array<TransactionInstruction> = [];
  for (let idx = 0; idx < buffers.length; ++idx) {
    if (checkExists && infos[idx] === null) continue;
    ixs.push(new TransactionInstruction({
      programId: CURVE_DALEK_ONCHAIN_PROGRAM_ID,
      keys: [
        {
          pubkey: buffers[idx],
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: walletKey,
          isSigner: true,
          isWritable: false,
        },
        {
          pubkey: SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ],
      data: Buffer.from([
        5,  // CloseBuffer...
      ]),
    }));
  }
  return ixs;
}

const initTransferIxs = async (
  connection: Connection,
  wasm: WasmConfig,
  walletKey: PublicKey,
  mintKey: PublicKey,
  transferBufferKeypair: Keypair,
  recipientKey: PublicKey,
) => {
  const transferBufferLen = wasm.transferBufferLen();
  const lamports = await connection.getMinimumBalanceForRentExemption(
      transferBufferLen);

  const [walletATAKey, ] = await PublicKey.findProgramAddress(
    [
      walletKey.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      mintKey.toBuffer(),
    ],
    SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
  );

  const privateMetadataKey = await getPrivateMetadata(mintKey);

  const instructions = [
    SystemProgram.createAccount({
      fromPubkey: walletKey,
      lamports,
      newAccountPubkey: transferBufferKeypair.publicKey,
      programId: PRIVATE_METADATA_PROGRAM_ID,
      space: transferBufferLen,
    }),
    // InitTransfer
    new TransactionInstruction({
      programId: PRIVATE_METADATA_PROGRAM_ID,
      keys: [
        {
          pubkey: walletKey,
          isSigner: true,
          isWritable: false,
        },
        {
          pubkey: mintKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: walletATAKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: privateMetadataKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: recipientKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: await getElgamalPubkeyAddress(recipientKey, mintKey),
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: transferBufferKeypair.publicKey,
          isSigner: true,
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
        1,
      ])
    }),
  ];

  return instructions;
}

const finiTransferIxs = async (
  connection: Connection,
  walletKey: PublicKey,
  destKey: PublicKey,
  mintKey: PublicKey,
  transferBufferKeypair: Keypair,
) => {
  const [walletATAKey, ] = await PublicKey.findProgramAddress(
    [
      walletKey.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      mintKey.toBuffer(),
    ],
    SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
  );

  const [destATAKey, ] = await PublicKey.findProgramAddress(
    [
      destKey.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      mintKey.toBuffer(),
    ],
    SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
  );

  const privateMetadataKey = await getPrivateMetadata(mintKey);

  const instructions : Array<TransactionInstruction> = [];

  if (await connection.getAccountInfo(destATAKey) === null) {
    instructions.push(
      Token.createAssociatedTokenAccountInstruction(
        SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        mintKey,
        destATAKey,
        destKey,
        walletKey,
      ),
    );
  }

  instructions.push(
    Token.createTransferInstruction(
      TOKEN_PROGRAM_ID,
      walletATAKey,
      destATAKey,
      walletKey,
      [],
      1,
    ),
  );

  instructions.push(
    new TransactionInstruction({
      programId: PRIVATE_METADATA_PROGRAM_ID,
      keys: [
        {
          pubkey: walletKey,
          isSigner: true,
          isWritable: true,
        },
        {
          pubkey: privateMetadataKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: transferBufferKeypair.publicKey,
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
        2, // FiniTransfer
      ])
    }),
  );

  return instructions;
}

type TransferChunkSlowKeys = {
  walletKey: PublicKey,
  mintKey: PublicKey,
  transferBufferKeypair: Keypair,
  instructionBufferPubkey: PublicKey,
  inputBufferKeypair: Keypair,
  computeBufferKeypair: Keypair,
};

const transferChunkSlowVerify = async (
  keys: TransferChunkSlowKeys,
  transferDataBytes: Buffer,
) => {
  const privateMetadataKey = await getPrivateMetadata(keys.mintKey);
  return {
    instructions: [
      new TransactionInstruction({
        programId: PRIVATE_METADATA_PROGRAM_ID,
        keys: [
          {
            pubkey: keys.walletKey,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: privateMetadataKey,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: keys.transferBufferKeypair.publicKey,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: keys.instructionBufferPubkey,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: keys.inputBufferKeypair.publicKey,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: keys.computeBufferKeypair.publicKey,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
        ],
        data: Buffer.from([
          4,  // TransferChunkSlow...
          ...transferDataBytes,
        ]),
      }),
    ],
    signers: [keys.walletKey],
  };
}

const buildTransferChunkTxns = async (
  connection: Connection,
  wasm: WasmConfig,
  cipherKey: string, // base64
  elgamalKeypair: Object, // TODO: typing
  recipientElgamalPubkey: Buffer,
  privateMetadata: PrivateMetadataAccount,
  recentBlockhash: Blockhash,
  keys: TransferChunkSlowKeys,
) => {
  const transferChunkTxsRes = wasm.transferChunkTxs(
    elgamalKeypair,
    [...recipientElgamalPubkey],
    {bytes: [...privateMetadata.encryptedCipherKey]},
    [...Buffer.from(cipherKey, 'base64')],
    {
      payer: [...keys.walletKey.toBuffer()],
      instruction_buffer: [...keys.instructionBufferPubkey.toBuffer()],
      input_buffer: [...keys.inputBufferKeypair.publicKey.toBuffer()],
      compute_buffer: [...keys.computeBufferKeypair.publicKey.toBuffer()],
    },
  );

  if (transferChunkTxsRes.Err) {
    throw new Error(transferChunkTxsRes.Err);
  }

  const [transferChunkTxs, transferDataBytes] = transferChunkTxsRes.Ok;

  // fixup rent and keys
  for (const tx of transferChunkTxs) {
    for (let idx = 0; idx < tx.instructions.length; ++idx) {
      const ix = tx.instructions[idx];
      ix.programId = new PublicKey(ix.program_id);
      ix.keys = ix.accounts.map((m: any) => ({
        pubkey: new PublicKey(m.pubkey),
        isSigner: m.is_signer,
        isWritable: m.is_writable,
      }));

      delete ix.program_id;
      delete ix.accounts;

      if (!ix.programId.equals(SystemProgram.programId)) {
        continue;
      }
      const space = new BN(ix.data.slice(12, 20), 'le').toNumber();
      const lamports = await connection.getMinimumBalanceForRentExemption(space);
      ix.data.splice(4, 8, ...new BN(lamports).toArray('le', 8));
    }

    tx.signers = tx.signers.map((s: Array<number>) => new PublicKey(s));
  }

  // build the verification ix
  transferChunkTxs.push(await transferChunkSlowVerify(
    keys,
    transferDataBytes,
  ));

  transferChunkTxs.push({
    instructions: await ensureBuffersClosed(
      connection,
      keys.walletKey,
      [keys.inputBufferKeypair.publicKey, keys.computeBufferKeypair.publicKey],
      false,
    ),
    signers: [keys.walletKey],
  });

  console.log(transferChunkTxs);

  // sign this group...
  const txns: Array<Transaction> = [];
  for (const txParams of transferChunkTxs) {
    const transaction = new Transaction();
    for (const ix of txParams.instructions) {
      transaction.add(ix);
    }
    transaction.recentBlockhash = recentBlockhash;
    transaction.setSigners(...txParams.signers);

    for (const s of [keys.inputBufferKeypair, keys.computeBufferKeypair]) {
      if (txParams.signers.find((p: PublicKey) => s.publicKey.equals(p))) {
        transaction.partialSign(s);
      }
    }
    txns.push(transaction);
  }

  return txns;
}

const buildTransaction = (
  walletKey: PublicKey,
  instructions: TransactionInstruction[],
  signers: Keypair[],
  recentBlockhash: Blockhash,
) => {
  const transaction = new Transaction();
  instructions.forEach((instruction) => transaction.add(instruction));
  transaction.recentBlockhash = recentBlockhash;
  transaction.setSigners(
    // fee payed by the wallet owner
    walletKey,
    ...signers.map((s) => s.publicKey)
  );

  if (signers.length > 0) {
    transaction.partialSign(...signers);
  }

  return transaction;
}

const decryptImage = (
  encryptedImage: Buffer,
  cipherKey: Buffer,
) => {
  const AES_BLOCK_SIZE = 16;
  const iv = encryptedImage.slice(0, AES_BLOCK_SIZE);

  // expects a base64 encoded string by default (openSSL mode?)
  // also possible to give a `format: CryptoJS.format.Hex`
  const ciphertext = encryptedImage.slice(AES_BLOCK_SIZE).toString('base64');
  // this can be a string but I couldn't figure out which encoding it
  // wants so just make it a WordArray
  const cipherKeyWordArray
    = CryptoJS.enc.Base64.parse(cipherKey.toString('base64'));
  const ivWordArray
    = CryptoJS.enc.Base64.parse(iv.toString('base64'));

  const decrypted = CryptoJS.AES.decrypt(
    ciphertext,
    cipherKeyWordArray,
    { iv: ivWordArray },
  );

  return Buffer.from(decrypted.toString(), 'hex');
};

type TransactionAndStep = {
  transaction: Transaction,
  step: number,
};

const WaitingOverlay = (props: {
  step: number;
  visible: boolean;
}) => {
  const setIconForStep = (currentStep: number, componentStep: number) => {
    if (currentStep === componentStep) {
      return <LoadingOutlined />;
    }
    return null;
  };

  const { Step } = Steps;

  const content = (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        pointerEvents: 'auto',
        justifyContent: 'center',
      }}
    >
      <Card>
        <Steps direction="vertical" current={props.step}>
          <Step
            className={'white-description'}
            title="Building transfer transactions"
            icon={setIconForStep(props.step, 0)}
          />
          <Step
            className={'white-description'}
            title="Signing transfer transactions"
            description="Approve the transactions from your wallet"
            icon={setIconForStep(props.step, 1)}
          />
          <Step
            className={'white-description'}
            title="Initializing buffers"
            icon={setIconForStep(props.step, 2)}
          />
          <Step
            className={'white-description'}
            title="Validating transfer encryption"
            icon={setIconForStep(props.step, 3)}
          />
          <Step
            className={'white-description'}
            title="Finalizing transfer"
            icon={setIconForStep(props.step, 4)}
          />
          <Step
            className={'white-description'}
            title="Waiting for Final Confirmation"
            description="This will take a few seconds."
            icon={setIconForStep(props.step, 5)}
          />
        </Steps>
      </Card>
    </div>
  );

  return (
    <Modal
      centered
      modalRender={() => content}
      width={'100vw'}
      mask={false}
      visible={props.visible}
    ></Modal>
  );
};

export const StealthView = (
  props: RouteComponentProps<{}>,
) => {
  // contexts
  const connection = useConnection();
  const wallet = useWallet();
  const wasm = useWasmConfig();
  const { loading, setLoading } = useLoading();

  // nav inputs
  const query = props.location.search;
  const params = queryString.parse(query);
  const [mint, setMint] = React.useState(params.mint as string || '');

  // user inputs
  const [recipientPubkeyStr, setRecipientPubkey]
    = useLocalStorageState('recipientPubkey', '');
  const [transferBuffer, setTransferBuffer]
    = useLocalStorageState('transferBuffer', '');
  const [instructionBuffer, setInstructionBuffer]
    = useLocalStorageState('instructionBuffer', '4huNP6jLbA9FGwHhMbjYQFP57ZmH1y4xa3ipAJ7mERNM');
  const [inputBuffer, setInputBuffer]
    = useLocalStorageState('inputBuffer', '');
  const [computeBuffer, setComputeBuffer]
    = useLocalStorageState('computeBuffer', '');

  // async useEffect set
  const [publicMetadata, setPublicMetadata]
      = React.useState<Metadata | null>(null);
  const [publicImageManifest, setPublicImageManifest]
      = React.useState<any>({}); // Object...
  const [privateMetadata, setPrivateMetadata]
      = React.useState<PrivateMetadataAccount | null>(null);
  const [elgamalKeypairStr, setElgamalKeypairStr]
      = useLocalStorageState(`elgamalKeypair:${mint}`, '');
  const [cipherKey, setCipherKey]
      = useLocalStorageState(`cipherKey:${mint}`, '');
  const [decryptedImage, setDecryptedImage]
      = React.useState<Buffer | null>(null);
  const [transferring, setTransferring]
      = React.useState<boolean>(false);
  const [transferProgress, setTransferProgress]
      = React.useState<number>(0);

  const clearFetchedState = () => {
    setPublicMetadata(null);
    setPrivateMetadata(null);
    setElgamalKeypairStr('');
    setCipherKey('');
    setDecryptedImage(null);
  };

  const parseAddress = (address: string): PublicKey | null => {
    try {
      return new PublicKey(address);
    } catch {
      return null;
    }
  };

  const parseKeypair = (secret: string): Keypair | null => {
    try {
      return Keypair.fromSecretKey(bs58.decode(secret));
    } catch {
      return null;
    }
  };

  React.useEffect(() => {
    if (wallet.disconnecting) {
      setCipherKey('');
      setDecryptedImage(null);
    }
  }, [wallet]);

  React.useEffect(() => {
    const mintKey = parseAddress(mint);
    if (mintKey === null) {
      clearFetchedState();
      return;
    }

    const wrap = async () => {
      const privateMetadataKey = await getPrivateMetadata(mintKey);
      const publicMetadataKey = await getMetadata(mintKey);

      const [privateMetadataAccount, publicMetadataAccount] =
        await connection.getMultipleAccountsInfo(
          [privateMetadataKey, publicMetadataKey]
        );

      if (privateMetadataAccount !== null) {
        const privateMetadata = decodePrivateMetadata(privateMetadataAccount.data);
        setPrivateMetadata(privateMetadata);
      }

      if (publicMetadataAccount !== null) {
        const publicMetadata = decodeMetadata(publicMetadataAccount.data);
        setPublicMetadata(publicMetadata);
      }
    };
    wrap();
  }, [connection, mint]);

  React.useEffect(() => {
    if (publicMetadata === null) return;
    const wrap = async () => {
      const response = await fetch(publicMetadata.data.uri);
      const manifest = await response.json();
      setPublicImageManifest(manifest);
    };
    wrap();
  }, [publicMetadata]);

  React.useEffect(() => {
    if (privateMetadata === null) return;
    const wrap = async () => {
      if (cipherKey) {
        const encryptedImage = Buffer.from(
          await (
            await fetch(privateMetadata.uri)
          ).arrayBuffer()
        );
        setDecryptedImage(decryptImage(encryptedImage, Buffer.from(cipherKey, 'base64')));
      }
    };
    wrap();
  }, [privateMetadata, cipherKey]);

  const notifyResult = (
    result: { txid: string } | string,
    name: string,
  ) => {
    if (typeof result === "string") {
      notify({
        message: `${name} failed`,
        description: result,
      });

      return null;
    } else {
      notify({
        message: `${name} succeeded`,
        description: (
          <a href={explorerLinkFor(result.txid, connection)}>
            View transaction on explorer
          </a>
        ),
      });

      return result.txid;
    }
  }

  const sendTransactionWithNotify = async (
    ixs: Array<TransactionInstruction>,
    signers: Array<Keypair>,
    name: string,
  ) => {
    const result = await sendTransactionWithRetry(
      connection,
      wallet,
      ixs,
      signers,
    );

    console.log(result);
    return notifyResult(result, name);
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
  const actualColumnWidth = (Math.min(width, maxWidth) - outerPadding - columnsGap * (cols - 1)) / cols;
  return (
    <div className="app stack" style={{ margin: 'auto' }}>
      <p
        className={"text-title"}
        style={{
          marginBottom: '15px',
        }}
      >
        {publicImageManifest?.name}
      </p>
      <div
        style={
          cols > 1
          ? {
            display: 'flex',
            flexDirection: 'row',
          }
          : {
            display: 'flex',
            flexDirection: 'column',
          }
        }
      >
        <CachedImageContent
          uri={publicImageManifest?.image}
          className={"fullAspectRatio"}
          style={{
            ...(cols > 1 ? { maxWidth: actualColumnWidth } : {}),
            minWidth: actualColumnWidth,
          }}
        />
        <div
          style={{
            ...(cols > 3 ? { paddingRight: '200px' } : {}),
            ...(
              cols > 1
              ? { paddingLeft: `${columnsGap}px` }
              : { paddingTop: '20px', paddingBottom: '20px', }
            ),
          }}
        >
          <div>
            <p
              className={"text-subtitle"}
              style={{
                fontSize: '15px',
                marginBottom: '10px',
              }}
            >
              {publicImageManifest?.description}
            </p>
          </div>
          <div>
            {publicImageManifest?.description
              && explorerLinkCForAddress(parseAddress(mint), connection)
            }
          </div>
        </div>
      </div>
      <Button
        style={{ width: '100%' }}
        className="metaplex-button"
        disabled={loading || !privateMetadata || !wallet.connected}
        onClick={() => {
          if (!privateMetadata || !wallet.connected) {
            return;
          }
          const mintKey = parseAddress(mint);
          if (mintKey === null) {
            console.error(`Failed to parse mint ${mint}`);
            return;
          }

          const run = async () => {
            const [cipherKey, elgamalKeypair] = await getCipherKey(
              connection, wallet, mintKey, privateMetadata, wasm);
            console.log(`Decoded cipher key bytes: ${[...cipherKey]}`);
            console.log(`Decoded cipher key: ${bs58.encode(cipherKey)}`);

            setElgamalKeypairStr(JSON.stringify(elgamalKeypair));
            setCipherKey(cipherKey.toString('base64'));
            notify({
              message: 'Decrypted cipher key',
            })
          };

          const wrap = async () => {
            try {
              await run();
            } catch (err) {
              console.error(err);
              notify({
                message: 'Failed to decrypt cipher key',
                description: err.message,
              })
            } finally {
              setLoading(false);
            }
          };

          setLoading(true);
          wrap();
        }}
      >
        Decrypt
      </Button>
      {decryptedImage && (
        <div>
          <DataUrlImageContent
            data={"data:image/png;base64," + decryptedImage.toString('base64')}
            style={{ maxWidth: '40ch', margin: 'auto', display: 'block' }}
          />
        </div>
      )}
      <label className="action-field">
        <span className="field-title">Recipient Pubkey</span>
        <Input
          id="recipient-pubkey-field"
          value={recipientPubkeyStr}
          onChange={(e) => setRecipientPubkey(e.target.value)}
          style={{ fontFamily: 'Monospace' }}
        />
      </label>
      <CollapsePanel
        id="transfer-options-collapse"
        panelName="Additional Options"
      >
        <label className="action-field">
          <span className="field-title">Instruction Buffer</span>
          <Input
            id="instruction-buffer-field"
            value={instructionBuffer}
            onChange={(e) => setInstructionBuffer(e.target.value)}
            style={{ fontFamily: 'Monospace' }}
          />
        </label>
        <label className="action-field">
          <span className="field-title">Transfer Buffer</span>
          <Input
            id="transfer-buffer-field"
            value={transferBuffer}
            onChange={(e) => setTransferBuffer(e.target.value)}
            style={{ fontFamily: 'Monospace' }}
          />
        </label>
        <label className="action-field">
          <span className="field-title">Input Buffer</span>
          <Input
            id="input-buffer-field"
            value={inputBuffer}
            onChange={(e) => setInputBuffer(e.target.value)}
            style={{ fontFamily: 'Monospace' }}
          />
        </label>
        <label className="action-field">
          <span className="field-title">Compute Buffer</span>
          <Input
            id="compute-buffer-field"
            value={computeBuffer}
            onChange={(e) => setComputeBuffer(e.target.value)}
            style={{ fontFamily: 'Monospace' }}
          />
        </label>
      </CollapsePanel>
      <Button
        style={{ width: '100%' }}
        className="metaplex-button"
        disabled={loading || !privateMetadata || !wallet.connected || !elgamalKeypairStr}
        onClick={() => {
          // TODO: requiring elgamalKeypair from decryption is a bit weird here...
          if (!privateMetadata || !wallet.connected || !elgamalKeypairStr) {
            return;
          }

          const validateKeypair = (secret: string, name: string) => {
            if (secret.length === 0) {
              return new Keypair();
            } else {
              const res = parseKeypair(secret);
              if (!res) {
                throw new Error(`Could not parse ${name} buffer key '${secret}'`);
              }
              return res;
            }
          }

          const run = async () => {
            const walletKey = wallet.publicKey;
            const mintKey = parseAddress(mint);
            if (!mintKey) {
              throw new Error(`Could not parse mint key '${mint}'`);
            }

            const instructionBufferPubkey = parseAddress(instructionBuffer);
            if (!instructionBufferPubkey) {
              throw new Error(`Could not parse instruction buffer key '${instructionBuffer}'`);
            }

            const inputBufferKeypair = validateKeypair(inputBuffer, 'input');
            const computeBufferKeypair = validateKeypair(computeBuffer, 'compute');
            const transferBufferKeypair = validateKeypair(transferBuffer, 'transfer');

            console.log('inputBufferKeypair', bs58.encode(inputBufferKeypair.secretKey));
            console.log('computeBufferKeypair', bs58.encode(computeBufferKeypair.secretKey));
            console.log('transferBufferKeypair', bs58.encode(transferBufferKeypair.secretKey));

            const recipientPubkey = new PublicKey(recipientPubkeyStr);
            if (recipientPubkey.equals(wallet.publicKey)) {
              throw new Error('Invalid transfer recipient (self)');
            }

            const recipientElgamalAddress = await getElgamalPubkeyAddress(recipientPubkey, mintKey);
            const recipientElgamalAccount = await connection.getAccountInfo(recipientElgamalAddress);
            if (recipientElgamalAccount === null) {
              throw new Error('Recipient has not yet published their elgamal pubkey for this mint');
            }

            const recipientElgamalPubkey = recipientElgamalAccount.data;
            const elgamalKeypair = JSON.parse(elgamalKeypairStr);

            const recentBlockhash = (
              await connection.getRecentBlockhash()
            ).blockhash;

            const transferTxns: Array<TransactionAndStep> = [];

            let transferBufferAccount = await connection.getAccountInfo(transferBufferKeypair.publicKey);
            let transferBufferUpdated;
            if (transferBufferAccount === null) {
              const createInstructions = await initTransferIxs(
                  connection, wasm, walletKey, mintKey, transferBufferKeypair, recipientPubkey);
              transferTxns.push({
                transaction: buildTransaction(
                  walletKey,
                  createInstructions,
                  [transferBufferKeypair],
                  recentBlockhash
                ),
                step: 2,
              });
              transferBufferUpdated = false;
            } else {
              const transferBufferDecoded = decodeTransferBuffer(transferBufferAccount.data);
              transferBufferUpdated = transferBufferDecoded.updated;
            }

            const closeInstructions = await ensureBuffersClosed(
              connection,
              walletKey,
              [inputBufferKeypair.publicKey, computeBufferKeypair.publicKey],
            );
            if (closeInstructions.length !== 0) {
              transferTxns.push({
                transaction: buildTransaction(
                  walletKey,
                  closeInstructions,
                  [],
                  recentBlockhash
                ),
                step: 2,
              });
            }

            if (!transferBufferUpdated) {
              const transferCrankTxns = await buildTransferChunkTxns(
                connection,
                wasm,
                cipherKey,
                elgamalKeypair,
                recipientElgamalPubkey,
                privateMetadata,
                recentBlockhash,
                {
                  walletKey,
                  mintKey,
                  transferBufferKeypair,
                  instructionBufferPubkey,
                  inputBufferKeypair,
                  computeBufferKeypair,
                },
              );
              transferTxns.push(...transferCrankTxns.map(
                transaction => ({
                  transaction,
                  step: 3,
                })
              ));
            }

            transferTxns.push({
              transaction: buildTransaction(
                walletKey,
                await finiTransferIxs(
                  connection,
                  walletKey,
                  recipientPubkey,
                  mintKey,
                  transferBufferKeypair,
                ),
                [],
                recentBlockhash
              ),
              step: 4,
            });

            console.log('Singing transactions...');
            let lastProgressStep = 1;
            setTransferProgress(lastProgressStep);
            const signedTxns = await wallet.signAllTransactions(transferTxns.map(t => t.transaction));
            for (let i = 0; i < signedTxns.length; ++i) {
              if (transferTxns[i].step != lastProgressStep) {
                lastProgressStep = transferTxns[i].step;
                setTransferProgress(lastProgressStep);
              }

              const resultTxid: TransactionSignature = await connection.sendRawTransaction(
                signedTxns[i].serialize(),
                {
                  skipPreflight: true,
                },
              );

              console.log('Waiting on confirmations for', resultTxid);

              let confirmed;
              if (i < signedTxns.length - 1) {
                confirmed = await connection.confirmTransaction(resultTxid, 'confirmed');
              } else {
                lastProgressStep += 1;
                setTransferProgress(lastProgressStep);
                confirmed = await connection.confirmTransaction(resultTxid, 'finalized');
              }

              console.log(confirmed);
              if (confirmed.value.err) {
                throw new Error('Crank failed. See console logs');
              }
            }
          };

          const wrap = async () => {
            try {
              await run();
            } catch (err) {
              console.error(err);
              notify({
                message: 'Failed to transfer NFT',
                description: err.message,
              })
            } finally {
              setTransferring(false);
            }
          };

          setTransferring(true);
          wrap();
        }}
      >
        Transfer
      </Button>
      <WaitingOverlay
        step={transferProgress}
        visible={transferring}
      />
    </div>
  );
}

