import * as React from "react";
import { BrowserRouter, Switch, Route } from 'react-router-dom';
import { hot } from "react-hot-loader";

import { CoingeckoProvider } from '../contexts/coingecko';
import { ConnectionProvider } from '../contexts/ConnectionContext';
import { SPLTokenListProvider } from '../contexts/tokenList';
import { WalletProvider } from '../contexts/WalletContext';
import { WasmProvider } from '../contexts/WasmContext';
import { AppLayout } from './Layout';

import { shortenAddress } from '../utils/common';
import { Layout, Tooltip } from 'antd';
import { CopyOutlined } from '@ant-design/icons';

const { Header, Content } = Layout;

import * as BN from 'bn.js';
import { WalletSigner } from "../contexts/WalletContext";
import { useWasmConfig, WasmConfig } from "../contexts/WasmContext";
import { sendTransactionWithRetry } from '../contexts/ConnectionContext';
import { wrap } from 'comlink';
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  TransactionSignature,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import * as bs58 from 'bs58';
import { explorerLinkFor, sendSignedTransaction } from '../utils/transactions';
import {
  decodePrivateMetadata,
  decodeTransferBuffer,
  PrivateMetadataAccount,
} from '../utils/privateSchema';
import {
  CURVE_DALEK_ONCHAIN_PROGRAM_ID,
  PRIVATE_METADATA_PROGRAM_ID,
  SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from '../utils/ids';
async function getPrivateMetadata(
  mint: PublicKey,
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        mint.toBuffer(),
      ],
      PRIVATE_METADATA_PROGRAM_ID,
    )
  )[0];
};

async function getElgamalKeypair(
  connection: Connection,
  wallet: WalletSigner,
  address: PublicKey,
  wasm: WasmConfig,
): Promise<any> { // TODO: type
  let transaction = new Transaction();
  transaction.add(new TransactionInstruction({
    programId: address, // mint
    keys: [],
    data: Buffer.from("ElGamalSecretKey"),
  }));

  const blockhash_bytes = 32;
  transaction.recentBlockhash = bs58.encode(
    new Array(blockhash_bytes).fill(0)
  );

  transaction.setSigners(wallet.publicKey);

  const signature = await wallet.signMessage(
      transaction.compileMessage().serialize());
  if (signature === null) {
    throw new Error(`Failed ElGamal keypair generation: signature`);
  }
  console.log('Signature {}', bs58.encode(signature));

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

  return [Buffer.concat(await Promise.all(privateMetadata.encryptedCipherKey.map(
    async (chunk) => {
      const decryptWorker = new Worker(new URL(
        '../utils/decryptWorker.js',
        import.meta.url,
      ));
      const decryptWorkerApi = wrap(decryptWorker) as any;
      console.log('Sending chunk to worker', chunk);
      const result: any = await decryptWorkerApi.decrypt(elgamalKeypair, chunk);
      if (result.Err) {
        throw new Error(result.Err);
      }
      return Buffer.from(result.Ok);
    }))
  ), elgamalKeypair];
}

const ensureBuffersClosed = async (
  connection: Connection,
  walletKey: PublicKey,
  buffers: Array<PublicKey>,
) => {
  const infos = await connection.getMultipleAccountsInfo(buffers);
  const ixs: Array<TransactionInstruction> = [];
  for (let idx = 0; idx < buffers.length; ++idx) {
    if (infos[idx] === null) continue;
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

const initializeTransferBuffer = async (
  connection: Connection,
  wasm: WasmConfig,
  walletKey: PublicKey,
  mintKey: PublicKey,
  transferBufferKeypair: Keypair,
  recipientElgamalPubkey: Buffer,
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
        ...recipientElgamalPubkey,
      ])
    }),
  ];

  return instructions;
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

import { Button, Input } from 'antd';
import { useWallet } from '@solana/wallet-adapter-react';
import { useConnection } from '../contexts/ConnectionContext';
import { useLocalStorageState } from '../utils/common';
import * as CryptoJS from 'crypto-js';
import { drawArray } from '../utils/image';
import { notify } from '../utils/common';
export const Demo = () => {
  // contexts
  const connection = useConnection();
  const wallet = useWallet();
  const wasm = useWasmConfig();

  // user inputs
  const [mint, setMint] = useLocalStorageState('mint', '');
  const [recipientElgamal, setRecipientElgamal]
    = useLocalStorageState('recipientElgamal', '');
  const [transferBuffer, setTransferBuffer]
    = useLocalStorageState('transferBuffer', '34BHKCEyS8zVTvEANG7W8uXcVTWoC2o7LbAdGiXezBFesFNMaGFf8fLYJYoK2p7ubiziwZjnTM7Ynk5fVC94AM2D');
  const [instructionBuffer, setInstructionBuffer]
    = useLocalStorageState('instructionBuffer', '4huNP6jLbA9FGwHhMbjYQFP57ZmH1y4xa3ipAJ7mERNM');
  const [inputBuffer, setInputBuffer]
    = useLocalStorageState('inputBuffer', '');
  const [computeBuffer, setComputeBuffer]
    = useLocalStorageState('computeBuffer', '');

  // async useEffect set
  const [privateMetadata, setPrivateMetadata]
      = React.useState<PrivateMetadataAccount | null>(null);
  const [elgamalKeypairStr, setElgamalKeypairStr]
      = useLocalStorageState('elgamalKeypair', '');
  const [cipherKey, setCipherKey]
      = useLocalStorageState('cipherKey', '');
  const [privateImage, setPrivateImage]
      = React.useState<Buffer | null>(null);
  const [decryptedImage, setDecryptedImage]
      = React.useState<Buffer | null>(null);

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
    const mintKey = parseAddress(mint);
    if (mintKey === null) return;

    const wrap = async () => {
      const privateMetadataKey = await getPrivateMetadata(mintKey);
      const privateMetadataAccount = await connection.getAccountInfo(privateMetadataKey);
      const privateMetadata = decodePrivateMetadata(privateMetadataAccount.data);

      setPrivateMetadata(privateMetadata);
    };
    wrap();
  }, [connection, mint]);

  React.useEffect(() => {
    if (privateMetadata === null) return;
    const wrap = async () => {
      let encryptedImage;
      if (!privateImage) {
        encryptedImage = Buffer.from(
          await (
            await fetch(privateMetadata.uri)
          ).arrayBuffer()
        );

        setPrivateImage(encryptedImage);
      } else {
        encryptedImage = privateImage;
      }

      if (cipherKey) {
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

  const { TextArea } = Input;

  return (
    <div className="app stack">
      <label className="action-field">
        <span className="field-title">NFT Mint</span>
        <Input
          id="mint-text-field"
          value={mint}
          onChange={(e) => setMint(e.target.value)}
          style={{ fontFamily: 'Monospace' }}
        />
      </label>
      {privateImage && (
        <div>
          <img
            style={{ margin: 'auto', display: 'block'}}
            src={"data:image/bmp;base64," + drawArray(privateImage, 8)}
          />
        </div>
      )}
      {decryptedImage && (
        <div>
          <img
            style={{ margin: 'auto', display: 'block'}}
            src={"data:image/png;base64," + decryptedImage.toString('base64')}
          />
        </div>
      )}
      <Button
        style={{ width: '100%' }}
        className="metaplex-button"
        disabled={!privateMetadata || !wallet?.connected}
        onClick={() => {
          if (!privateMetadata || !wallet?.connected) {
            return;
          }
          const mintKey = parseAddress(mint);
          if (mintKey === null) {
            console.error(`Failed to parse mint ${mint}`);
          }
          const run = async () => {
            const [cipherKey, elgamalKeypair] = await getCipherKey(
              connection, wallet, mintKey, privateMetadata, wasm);
            console.log(`Decoded cipher key bytes: ${[...cipherKey]}`);
            console.log(`Decoded cipher key: ${bs58.encode(cipherKey)}`);

            setElgamalKeypairStr(JSON.stringify(elgamalKeypair));
            setCipherKey(cipherKey.toString('base64'));
          }
          const wrap = async () => {
            try {
              await run();
            } catch (err) {
              // console.error(err);
              notify({
                message: 'Failed to decrypt cipher key',
                description: err.message,
              })
            }
          };
          wrap();
        }}
      >
        Decrypt
      </Button>
      <label className="action-field">
        <span className="field-title">Recipient ElGamal Pubkey</span>
        <Input
          id="recipient-elgamal-field"
          value={recipientElgamal}
          onChange={(e) => setRecipientElgamal(e.target.value)}
          style={{ fontFamily: 'Monospace' }}
        />
      </label>
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
        <TextArea
          rows={2}
          id="transfer-buffer-field"
          value={transferBuffer}
          onChange={(e) => setTransferBuffer(e.target.value)}
          style={{ fontFamily: 'Monospace' }}
        />
      </label>
      <label className="action-field">
        <span className="field-title">Input Buffer</span>
        <TextArea
          rows={2}
          id="input-buffer-field"
          value={inputBuffer}
          onChange={(e) => setInputBuffer(e.target.value)}
          style={{ fontFamily: 'Monospace' }}
        />
      </label>
      <label className="action-field">
        <span className="field-title">Compute Buffer</span>
        <TextArea
          rows={2}
          id="compute-buffer-field"
          value={computeBuffer}
          onChange={(e) => setComputeBuffer(e.target.value)}
          style={{ fontFamily: 'Monospace' }}
        />
      </label>
      <Button
        style={{ width: '100%' }}
        className="metaplex-button"
        disabled={!privateMetadata || !wallet?.connected || !elgamalKeypairStr}
        onClick={() => {
          // TODO: requiring elgamalKeypair from decryption is a bit weird here...
          if (!privateMetadata || !wallet?.connected || !elgamalKeypairStr) {
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

          const wrap = async () => {
            try {
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

              const recipientElgamalPubkey = Buffer.from(recipientElgamal, 'base64');
              if (recipientElgamalPubkey.length !== 32) {
                throw new Error('Recipient elgamal pubkey does not look correct');
              }

              let transferBufferAccount = await connection.getAccountInfo(transferBufferKeypair.publicKey);
              if (transferBufferAccount === null) {
                const createInstructions = await initializeTransferBuffer(
                    connection, wasm, walletKey, mintKey, transferBufferKeypair, recipientElgamalPubkey);

                const createTxid = await sendTransactionWithNotify(
                  createInstructions,
                  [transferBufferKeypair],
                  "Transfer buffer create",
                );

                await connection.confirmTransaction(createTxid, "confirmed");

                transferBufferAccount = await connection.getAccountInfo(transferBufferKeypair.publicKey);
              }

              const closeIxs = await ensureBuffersClosed(
                connection,
                walletKey,
                [inputBufferKeypair.publicKey, computeBufferKeypair.publicKey],
              );
              if (closeIxs.length !== 0) {
                const closeTxid = await sendTransactionWithNotify(
                  closeIxs,
                  [],
                  "Input and compute buffer close",
                );

                await connection.confirmTransaction(closeTxid, "confirmed");

              }

              const transferBufferDecoded = decodeTransferBuffer(transferBufferAccount.data);
              if (transferBufferDecoded.updated !== 0) {
                return;
              }

              const chunk = 0;
              const keychunk = Buffer.from(cipherKey, 'base64')
                  .slice(chunk * 4, (chunk + 1) * 4);

              const elgamalKeypair = JSON.parse(elgamalKeypairStr);

              console.log(elgamalKeypair);

              const transferChunkTxsRes = wasm.transferChunkTxs(
                elgamalKeypair,
                [...recipientElgamalPubkey],
                {bytes: [...privateMetadata.encryptedCipherKey[chunk]]},
                // TODO: pass buffer and convert on rust side?
                new BN(keychunk, 'le').toNumber(),
                {
                  payer: [...walletKey.toBuffer()],
                  instruction_buffer: [...instructionBufferPubkey.toBuffer()],
                  input_buffer: [...inputBufferKeypair.publicKey.toBuffer()],
                  compute_buffer: [...computeBufferKeypair.publicKey.toBuffer()],
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
              const privateMetadataKey = await getPrivateMetadata(mintKey);
              transferChunkTxs.push({
                instructions: [
                  new TransactionInstruction({
                    programId: PRIVATE_METADATA_PROGRAM_ID,
                    keys: [
                      {
                        pubkey: walletKey,
                        isSigner: true,
                        isWritable: false,
                      },
                      {
                        pubkey: privateMetadataKey,
                        isSigner: false,
                        isWritable: false,
                      },
                      {
                        pubkey: transferBufferKeypair.publicKey,
                        isSigner: false,
                        isWritable: true,
                      },
                      {
                        pubkey: instructionBufferPubkey,
                        isSigner: false,
                        isWritable: false,
                      },
                      {
                        pubkey: inputBufferKeypair.publicKey,
                        isSigner: false,
                        isWritable: false,
                      },
                      {
                        pubkey: computeBufferKeypair.publicKey,
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
                      chunk,
                      ...transferDataBytes,
                    ]),
                  }),
                ],
                signers: [walletKey],
              })

              console.log(transferChunkTxs);

              // sign this group...
              const recentBlockhash = (
                await connection.getRecentBlockhash()
              ).blockhash;
              const txns: Array<Transaction> = [];
              for (const txParams of transferChunkTxs) {
                const transaction = new Transaction();
                for (const ix of txParams.instructions) {
                  transaction.add(ix);
                }
                transaction.recentBlockhash = recentBlockhash;
                transaction.setSigners(...txParams.signers);

                for (const s of [inputBufferKeypair, computeBufferKeypair]) {
                  if (txParams.signers.find((p: PublicKey) => s.publicKey.equals(p))) {
                    transaction.partialSign(s);
                  }
                }
                txns.push(transaction);
              }

              console.log('Singing transactions...');
              const signedTxns = await wallet.signAllTransactions(txns);
              for (let i = 0; i < signedTxns.length; ++i) {
                const resultTxid: TransactionSignature = await connection.sendRawTransaction(
                  signedTxns[i].serialize(),
                  {
                    skipPreflight: true,
                  },
                );

                console.log('Waiting on confirmations for', resultTxid);

                const confirmed = await connection.confirmTransaction(
                    resultTxid, "confirmed");

                console.log(confirmed);
                notifyResult(
                  confirmed.value.err ? 'See console logs' : {txid: resultTxid},
                  `Transfer crank ${i + 1} of ${signedTxns.length}`,
                );
              }

            } catch (err) {
              notify({
                message: 'Failed to transfer NFT',
                description: err.message,
              })
            }
          };
          wrap();
        }}
      >
        Transfer
      </Button>
    </div>
  );
}

export const App = () => {
  return (
    <BrowserRouter>
      <WasmProvider>
      <ConnectionProvider>
      <WalletProvider>
      <SPLTokenListProvider>
      <CoingeckoProvider>
        <AppLayout>
          <Switch>
            <Route path="/" component={() => (
              <Demo />
            )} />
          </Switch>
        </AppLayout>
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
