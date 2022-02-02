import * as React from "react";
import { RouteComponentProps } from 'react-router-dom';
import queryString from 'query-string';

import {
  Button,
  Input,
  Progress,
  Steps,
} from 'antd';
import {
  LoadingOutlined,
} from '@ant-design/icons';
import { useWallet } from '@solana/wallet-adapter-react';
import {
  Blockhash,
  Connection,
  Keypair,
  PublicKey,
  SignatureStatus,
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
import {
  useLoading,
  incLoading,
  decLoading,
} from '../../components/Loader';
import { MetaplexModal } from '../../components/MetaplexModal';
import { useWindowDimensions } from '../../components/AppBar';
import {
  CachedImageContent,
} from '../../components/ArtContent';
import {
  explorerLinkCForAddress,
  useConnection,
} from '../../contexts/ConnectionContext';
import { WalletSigner } from "../../contexts/WalletContext";
import {
  getElgamalKeypair,
  useWasmConfig,
  WasmConfig,
} from "../../contexts/WasmContext";
import {
  notify,
  sleep,
  useLocalStorageState,
} from '../../utils/common';
import {
  decodeEncryptionKeyBuffer,
  decodeStealth,
  decodeTransferBuffer,
  StealthAccount,
} from '../../utils/stealthSchema';
import {
  decodeMetadata,
  Metadata,
} from '../../utils/publicSchema';
import {
  getElgamalPubkeyAddress,
  getMetadata,
  getStealth,
  getTransferBufferAddress,
  parseAddress,
  parseKeypair,
  CURVE_DALEK_ONCHAIN_PROGRAM_ID,
  STEALTH_PROGRAM_ID,
  SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from '../../utils/ids';

async function getCipherKey(
  wallet: WalletSigner,
  address: PublicKey,
  privateMetadata: StealthAccount,
  wasm: WasmConfig,
): Promise<[Buffer, Buffer]> {
  const elgamalKeypairRes = await getElgamalKeypair(
    wallet, address, wasm);

  if (elgamalKeypairRes.Err) {
    throw new Error(elgamalKeypairRes.Err);
  }

  const elgamalKeypair = elgamalKeypairRes.Ok;

  const result = wasm.elgamalDecrypt(
      elgamalKeypair, { bytes: [...privateMetadata.encryptedCipherKey] });
  if (result.Err) {
    console.error('Failed decrypt', result.Err, privateMetadata.encryptedCipherKey);
    throw new Error(result.Err);
  }
  const decrypted = Buffer.from(result.Ok);

  return [decrypted, elgamalKeypair];
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
  walletKey: PublicKey,
  mintKey: PublicKey,
  transferBufferPubkey: PublicKey,
  recipientKey: PublicKey,
) => {
  const [walletATAKey, ] = await PublicKey.findProgramAddress(
    [
      walletKey.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      mintKey.toBuffer(),
    ],
    SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
  );

  const privateMetadataKey = await getStealth(mintKey);

  const instructions = [
    // InitTransfer
    new TransactionInstruction({
      programId: STEALTH_PROGRAM_ID,
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
          pubkey: transferBufferPubkey,
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
  transferBufferPubkey: PublicKey
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

  const privateMetadataKey = await getStealth(mintKey);

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
    new TransactionInstruction({
      programId: STEALTH_PROGRAM_ID,
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
          pubkey: transferBufferPubkey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: SystemProgram.programId,
          isSigner: false,
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
          isWritable: true,
        },
        {
          pubkey: destATAKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: TOKEN_PROGRAM_ID,
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
  transferBufferPubkey: PublicKey,
  instructionBufferPubkey: PublicKey,
  inputBufferKeypair: Keypair,
  computeBufferKeypair: Keypair,
};

const transferChunkSlowVerify = async (
  keys: TransferChunkSlowKeys,
  transferDataBytes: Buffer,
) => {
  const privateMetadataKey = await getStealth(keys.mintKey);
  return {
    instructions: [
      new TransactionInstruction({
        programId: STEALTH_PROGRAM_ID,
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
            pubkey: keys.transferBufferPubkey,
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
  privateMetadata: StealthAccount,
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

type TransferProgress = {
  step: number,
  substep?: number,
};

type TransactionAndProgress = {
  transaction: Transaction,
  progress: TransferProgress
};

const WaitingOverlay = (props: {
  visible: boolean;
  progress: TransferProgress,
}) => {
  const setIconForStep = (currentStep: number, componentStep: number) => {
    if (currentStep === componentStep) {
      return <LoadingOutlined />;
    }
    return null;
  };

  const { Step } = Steps;
  const { step, substep } = props.progress;

  // closes after end of async function
  return (
    <MetaplexModal
      visible={props.visible}
      closable={false}
    >
      <Steps direction="vertical" current={step}>
        <Step
          className={'white-description'}
          title="Building transfer transactions"
          icon={setIconForStep(step, 0)}
        />
        <Step
          className={'white-description'}
          title="Signing transfer transactions"
          description="Approve the transactions from your wallet"
          icon={setIconForStep(step, 1)}
        />
        <Step
          className={'white-description'}
          title="Initializing buffers"
          icon={setIconForStep(step, 2)}
        />
        <Step
          className={'white-description'}
          title="Sending transfer encryption"
          description=
            {step === 3 && <Progress percent={substep} />}
          icon={setIconForStep(step, 3)}
        />
        <Step
          className={'white-description'}
          title="Confirming transfer encryption"
          description=
            {step === 4 && <Progress percent={substep} />}
          icon={setIconForStep(step, 4)}
        />
        <Step
          className={'white-description'}
          title="Finalizing transfer"
          icon={setIconForStep(step, 5)}
        />
        <Step
          className={'white-description'}
          title="Waiting for Final Confirmation"
          description="This will take a few seconds."
          icon={setIconForStep(step, 6)}
        />
      </Steps>
    </MetaplexModal>
  );
};

type AssetAndType = {
  uri: string,
  type: string,
};

type PublicImageManifest = {
  name: string,
  image: string,
  description: string,
  // others...
};

type PrivateImageManifest = {
  name: string,
  cover_image: AssetAndType,
  encrypted_assets: Array<AssetAndType>,
  encrypted_blob: string,
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
  const mint = params.mint as string || '';

  // user inputs
  const [recipientPubkeyStr, setRecipientPubkey]
    = useLocalStorageState('recipientPubkey', '');
  const [instructionBuffer, setInstructionBuffer]
    = useLocalStorageState('instructionBuffer', '4X5dmqKWQojDNAqgV3JToRa5uCDFwijSMWj8zHDXpQ9g');
  const [inputBuffer, setInputBuffer]
    = useLocalStorageState('inputBuffer', '');
  const [computeBuffer, setComputeBuffer]
    = useLocalStorageState('computeBuffer', '');

  // async useEffect set
  const [publicMetadata, setPublicMetadata]
      = React.useState<Metadata | null>(null);
  const [publicImageManifest, setPublicImageManifest]
      = React.useState<PublicImageManifest | null>(null);
  const [privateMetadata, setPrivateMetadata]
      = React.useState<StealthAccount | null>(null);
  const [privateImageManifest, setPrivateImageManifest]
      = React.useState<PrivateImageManifest| null>(null);
  const [elgamalKeypairStr, setElgamalKeypairStr]
      = useLocalStorageState(`elgamalKeypair:${mint}`, '');
  const [cipherKey, setCipherKey]
      = useLocalStorageState(`cipherKey:${mint}`, '');
  const [decryptedImage, setDecryptedImage]
      = React.useState<Buffer | null>(null);
  const [transferInputting, setTransferInputting]
      = React.useState<boolean>(false);
  const [transferring, setTransferring]
      = React.useState<boolean>(false);
  const [transferProgress, setTransferProgress]
      = React.useState<TransferProgress>({ step: 0 });

  const clearPrivateState = () => {
    setElgamalKeypairStr('');
    setCipherKey('');
    setDecryptedImage(null);
  };

  React.useEffect(() => {
    if (wallet.disconnecting) {
      clearPrivateState();
    }
  }, [wallet]);

  const [toggleRunFetchMetadata, setRunFetchMetadata]
      = React.useState<boolean>(false);
  React.useEffect(() => {
    const mintKey = parseAddress(mint);
    if (mintKey === null) return;

    const wrap = async () => {
      const privateMetadataKey = await getStealth(mintKey);
      const publicMetadataKey = await getMetadata(mintKey);

      const [privateMetadataAccount, publicMetadataAccount] =
        await connection.getMultipleAccountsInfo(
          [privateMetadataKey, publicMetadataKey]
        );

      if (privateMetadataAccount !== null) {
        const privateMetadata = decodeStealth(privateMetadataAccount.data);
        setPrivateMetadata(privateMetadata);
      } else {
        setLoading(decLoading);
      }

      if (publicMetadataAccount !== null) {
        const publicMetadata = decodeMetadata(publicMetadataAccount.data);
        setPublicMetadata(publicMetadata);
      } else {
        setLoading(decLoading);
      }
    };

    setLoading(p => incLoading(incLoading(p)));
    wrap();
  }, [connection, mint, toggleRunFetchMetadata]);
  const runFetchMetadata = () => setRunFetchMetadata(!toggleRunFetchMetadata);

  React.useEffect(() => {
    if (publicMetadata === null) return;
    const wrap = async () => {
      const response = await fetch(publicMetadata.data.uri);
      const manifest = await response.json();
      setPublicImageManifest(manifest);
      setLoading(decLoading);
    };
    wrap();
  }, [publicMetadata]);

  React.useEffect(() => {
    if (privateMetadata === null) return;
    const wrap = async () => {
      const response = await fetch(privateMetadata.uri);
      const manifest = await response.json();
      setPrivateImageManifest(manifest);
      setLoading(decLoading);
    };
    wrap();
  }, [privateMetadata]);

  React.useEffect(() => {
    if (privateImageManifest === null) return;
    const wrap = async () => {
      if (cipherKey) {
        const encryptedImage = Buffer.from(
          await (
            await fetch(privateImageManifest.encrypted_assets[0].uri)
          ).arrayBuffer()
        );
        setDecryptedImage(decryptImage(encryptedImage, Buffer.from(cipherKey, 'base64')));
      }
    };
    wrap();
  }, [privateImageManifest, cipherKey]);

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

    console.log('inputBufferKeypair', bs58.encode(inputBufferKeypair.secretKey));
    console.log('computeBufferKeypair', bs58.encode(computeBufferKeypair.secretKey));

    const recipientPubkey = new PublicKey(recipientPubkeyStr);
    if (recipientPubkey.equals(wallet.publicKey)) {
      throw new Error('Invalid transfer recipient (self)');
    }

    const recipientElgamalAddress = await getElgamalPubkeyAddress(recipientPubkey, mintKey);
    const recipientElgamalAccount = await connection.getAccountInfo(recipientElgamalAddress);
    if (recipientElgamalAccount === null) {
      throw new Error('Recipient has not yet published their elgamal pubkey for this mint');
    }

    const recipientElgamalPubkey = Buffer.from(decodeEncryptionKeyBuffer(recipientElgamalAccount.data).elgamalPk);
    const elgamalKeypair = JSON.parse(elgamalKeypairStr);

    const recentBlockhash = (
      await connection.getRecentBlockhash()
    ).blockhash;

    const transferTxns: Array<TransactionAndProgress> = [];

    const transferBufferPubkey = await getTransferBufferAddress(recipientPubkey, mintKey);
    let transferBufferAccount = await connection.getAccountInfo(transferBufferPubkey);
    let transferBufferUpdated;
    if (transferBufferAccount === null) {
      const createInstructions = await initTransferIxs(
          walletKey, mintKey, transferBufferPubkey, recipientPubkey);
      transferTxns.push({
        transaction: buildTransaction(
          walletKey,
          createInstructions,
          [],
          recentBlockhash
        ),
        progress: { step: 2 },
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
        progress: { step: 2 },
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
          transferBufferPubkey,
          instructionBufferPubkey,
          inputBufferKeypair,
          computeBufferKeypair,
        },
      );
      transferTxns.push(...transferCrankTxns.map(
        (transaction, idx) => ({
          transaction,
          progress: {
            step: 3,
            substep: Math.floor(idx * 100 / transferCrankTxns.length),
          },
        })
      ));
    }

  // 'skip' step 4 of confirming...

    transferTxns.push({
      transaction: buildTransaction(
        walletKey,
        await finiTransferIxs(
          connection,
          walletKey,
          recipientPubkey,
          mintKey,
          transferBufferPubkey,
        ),
        [],
        recentBlockhash
      ),
      progress: { step: 5 },
    });

    console.log('Singing transactions...');
    let lastProgress: TransferProgress = { step: 1 };
    let crankTransactions = 0;
    let setupCrankTransactions = 2;
    let pendingCrankSignatures: Array<string> = [];
    setTransferProgress(lastProgress);
    const signedTxns = await wallet.signAllTransactions(transferTxns.map(t => t.transaction));
    for (let i = 0; i < signedTxns.length; ++i) {
      const curProgress = transferTxns[i].progress;

      // bespoke crank handling
      if (curProgress.step === 5 && pendingCrankSignatures.length > 0) {
        while (true) {
          const statuses = await connection.getSignatureStatuses(pendingCrankSignatures);
          console.log('Waiting on confirmations for', pendingCrankSignatures, statuses);
          const confirmedSigs = statuses.value.filter((v: null | SignatureStatus) => {
            if (v === null) return false;
            return v.confirmationStatus === "confirmed";
          }).length;

          setTransferProgress({
            step: 4,
            substep: Math.floor((confirmedSigs + setupCrankTransactions) * 100 / crankTransactions),
          });

          if (confirmedSigs === pendingCrankSignatures.length) {
            break;
          }
          await sleep(1000);
        }
        pendingCrankSignatures = [];
      }

      if (curProgress.step != lastProgress.step
          || curProgress.substep != lastProgress.substep) {
        lastProgress = curProgress;
        setTransferProgress(lastProgress);
      }

      const resultTxid: TransactionSignature = await connection.sendRawTransaction(
        signedTxns[i].serialize(),
        {
          skipPreflight: true,
        },
      );

      // bespoke crank handling
      if (curProgress.step === 3) {
        crankTransactions += 1;
        if (crankTransactions > setupCrankTransactions) {
          console.log('Deferring wait for', resultTxid);
          pendingCrankSignatures.push(resultTxid);
          continue;
        }
      }

      console.log('Waiting on confirmations for', resultTxid);

      let confirmed;
      if (i < signedTxns.length - 1) {
        confirmed = await connection.confirmTransaction(resultTxid, 'confirmed');
      } else {
        setTransferProgress({ step: 6 });
        confirmed = await connection.confirmTransaction(resultTxid, 'finalized');
        setTransferProgress({ step: 7 });
      }

      console.log(confirmed);
      if (confirmed.value.err) {
        throw new Error('Crank failed. See console logs');
      }
    }
  };


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
  const actualColumnWidth = (Math.min(width, maxWidth) - outerPadding - columnsGap * (cols - 1)) / cols;

  const publicPreviewC = () => (
    <React.Fragment>
      <p
        className={"text-title"}
        style={{
          marginBottom: '15px',
        }}
      >
        {publicImageManifest.name}
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
          uri={publicImageManifest.image}
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
              {publicImageManifest.description}
            </p>
          </div>
          <div>
            {publicImageManifest.description
              && explorerLinkCForAddress(parseAddress(mint).toBase58(), connection)
            }
          </div>
        </div>
      </div>
    </React.Fragment>
  );

  if (publicImageManifest === null) {
    return null;
  }

  if (privateImageManifest === null) {
    return publicPreviewC();
  }

  return (
    <div className="app stack" style={{ margin: 'auto' }}>
      {publicPreviewC()}
      <p
        className={"text-title"}
        style={{
          marginBottom: '15px',
        }}
      >
        stealthed assets
      </p>
      {decryptedImage ? (
        <div>
          {privateImageManifest.encrypted_assets[0].type === "video/mp4"
            // TODO: less janky / hardcoded
            ? (
              <video
                controls
              >
                <source
                  type="video/mp4"
                  src={"data:video/mp4;base64," + decryptedImage.toString('base64')}
                />
              </video>
            )
            : (
              <img
                src={"data:video/mp4;base64," + decryptedImage.toString('base64')}
                style={{ maxWidth: '40ch', margin: 'auto', display: 'block' }}
              />
            )
          }
        </div>
      ) : (
        <React.Fragment>
          <CachedImageContent
            uri={privateImageManifest && privateImageManifest.cover_image.uri}
            className={"fullAspectRatio"}
            style={{
              ...(cols > 1 ? { maxWidth: actualColumnWidth } : {}),
              minWidth: actualColumnWidth,
            }}
          />
        </React.Fragment>
      )}
      <Button
        style={{ width: '100%' }}
        className="metaplex-button"
        disabled={!!loading || !privateMetadata || !wallet.connected}
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
              wallet, mintKey, privateMetadata, wasm);
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
              setLoading(decLoading);
            }
          };

          setLoading(incLoading);
          wrap();
        }}
      >
        Decrypt
      </Button>
      <Button
        style={{ width: '100%' }}
        className="metaplex-button"
        disabled={!!loading || !privateMetadata || !wallet.connected || !elgamalKeypairStr || !publicImageManifest?.name}
        onClick={() => setTransferInputting(true)}
      >
        Transfer
      </Button>
      <MetaplexModal
        title={`Send ${publicImageManifest?.name}`}
        visible={transferInputting}
        onCancel={() => setTransferInputting(false)}
      >
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
          className="metaplex-button"
          onClick={() => {
            const wrap = async () => {
              try {
                await run();
                clearPrivateState();
                runFetchMetadata();
                notify({
                  message: 'Transfer complete',
                })
              } catch (err) {
                console.error(err);
                notify({
                  message: 'Failed to transfer NFT',
                  description: err.message,
                })
              } finally {
                setTransferring(false);
                setTransferProgress({ step: 0 });
              }
            };

            setTransferInputting(false);
            setTransferring(true);
            wrap();
          }}
        >
          Transfer
        </Button>
      </MetaplexModal>
      <WaitingOverlay
        visible={transferring}
        progress={transferProgress}
      />
    </div>
  );
}

