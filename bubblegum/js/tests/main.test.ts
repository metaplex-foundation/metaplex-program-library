import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';

import {
  getConcurrentMerkleTreeAccountSize,
  createVerifyLeafIx,
  ConcurrentMerkleTreeAccount,
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  SPL_NOOP_PROGRAM_ID,
  ValidDepthSizePair
} from '@solana/spl-account-compression';

import {
  createCreateTreeInstruction,
  createMintV1Instruction,
  createTransferInstruction,
  createBurnInstruction,
  createRedeemInstruction,
  createDecompressV1Instruction,
  MetadataArgs,
  PROGRAM_ID as BUBBLEGUM_PROGRAM_ID,
  TokenProgramVersion,
  TokenStandard,
  Creator,
} from '../src/generated';
import { getLeafAssetId, computeDataHash, computeCreatorHash, computeCompressedNFTHash } from '../src/mpl-bubblegum';
import { BN } from 'bn.js';
import { PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token";

function keypairFromSeed(seed: string) {
  const expandedSeed = Uint8Array.from(
    Buffer.from(`${seed}`),
  );
  return Keypair.fromSeed(expandedSeed.slice(0, 32));
}

function makeCompressedNFT(name: string, symbol: string, creators: Creator[] = []): MetadataArgs {
  return {
    name: name,
    symbol: symbol,
    uri: 'https://metaplex.com',
    creators,
    editionNonce: 0,
    tokenProgramVersion: TokenProgramVersion.Original,
    tokenStandard: TokenStandard.Fungible,
    uses: null,
    collection: null,
    primarySaleHappened: false,
    sellerFeeBasisPoints: 0,
    isMutable: false,
  };
}

async function setupTreeWithCompressedNFT(
  connection: Connection,
  payerKeypair: Keypair,
  compressedNFT: MetadataArgs,
  depthSizePair: ValidDepthSizePair = {
    maxDepth: 14,
    maxBufferSize: 64
  }
): Promise<{
  merkleTree: PublicKey;
}> {
  const payer = payerKeypair.publicKey;

  const merkleTreeKeypair = Keypair.generate();
  const merkleTree = merkleTreeKeypair.publicKey;
  const space = getConcurrentMerkleTreeAccountSize(depthSizePair.maxDepth, depthSizePair.maxBufferSize);
  const allocTreeIx = SystemProgram.createAccount({
    fromPubkey: payer,
    newAccountPubkey: merkleTree,
    lamports: await connection.getMinimumBalanceForRentExemption(space),
    space: space,
    programId: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  });
  const [treeAuthority, _bump] = await PublicKey.findProgramAddress(
    [merkleTree.toBuffer()],
    BUBBLEGUM_PROGRAM_ID,
  );
  const createTreeIx = createCreateTreeInstruction(
    {
      merkleTree,
      treeAuthority,
      treeCreator: payer,
      payer,
      logWrapper: SPL_NOOP_PROGRAM_ID,
      compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
    },
    {
      maxBufferSize: depthSizePair.maxBufferSize,
      maxDepth: depthSizePair.maxDepth,
      public: false,
    },
    BUBBLEGUM_PROGRAM_ID,
  );

  const mintIx = createMintV1Instruction(
    {
      merkleTree,
      treeAuthority,
      treeDelegate: payer,
      payer,
      leafDelegate: payer,
      leafOwner: payer,
      compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
      logWrapper: SPL_NOOP_PROGRAM_ID,
    },
    {
      message: compressedNFT,
    },
  );

  const tx = new Transaction().add(allocTreeIx).add(createTreeIx).add(mintIx);
  tx.feePayer = payer;
  await sendAndConfirmTransaction(connection, tx, [merkleTreeKeypair, payerKeypair], {
    commitment: 'confirmed',
    skipPreflight: true,
  });

  return {
    merkleTree,
  };
}

describe('Bubblegum tests', () => {
  const connection = new Connection('http://localhost:8899');
  const payerKeypair = keypairFromSeed('metaplex-test09870987098709870987009709870987098709870987');
  const payer = payerKeypair.publicKey;

  beforeEach(async () => {
    await connection.requestAirdrop(payer, LAMPORTS_PER_SOL);
  });
  it('Can create a Bubblegum tree and mint to it', async () => {
    const compressedNFT: MetadataArgs = {
      name: 'Test Compressed NFT',
      symbol: 'TST',
      uri: 'https://metaplex.com',
      creators: [],
      editionNonce: 0,
      tokenProgramVersion: TokenProgramVersion.Original,
      tokenStandard: TokenStandard.Fungible,
      uses: null,
      collection: null,
      primarySaleHappened: false,
      sellerFeeBasisPoints: 0,
      isMutable: false,
    };
    await setupTreeWithCompressedNFT(connection, payerKeypair, compressedNFT, { maxDepth: 14, maxBufferSize: 64 });
  });

  describe('Unit test compressed NFT instructions', () => {
    let merkleTree: PublicKey;
    let creators: Creator[] = [
      {
        address: payer,
        share: 55,
        verified: false,
      },
      {
        address: new Keypair().publicKey,
        share: 45,
        verified: false,
      },
    ]
    const originalCompressedNFT = makeCompressedNFT('test', 'TST', creators);
    beforeEach(async () => {
      await connection.requestAirdrop(payer, LAMPORTS_PER_SOL);
      const result = await setupTreeWithCompressedNFT(
        connection,
        payerKeypair,
        originalCompressedNFT,
        {
          maxDepth: 14,
          maxBufferSize: 64,
        }
      );
      merkleTree = result.merkleTree;
    });
    it('Can verify existence a compressed NFT', async () => {
      // Todo(@ngundotra): expose commitment level in ConcurrentMerkleTreeAccount.fromAddress
      const accountInfo = await connection.getAccountInfo(merkleTree, { commitment: 'confirmed' });
      const account = ConcurrentMerkleTreeAccount.fromBuffer(accountInfo!.data!);

      // Verify leaf exists.
      const leafIndex = new BN.BN(0);
      const assetId = await getLeafAssetId(merkleTree, leafIndex);
      const verifyLeafIx = createVerifyLeafIx(
        merkleTree,
        {
          root: account.getCurrentRoot(),
          leaf: computeCompressedNFTHash(assetId, payer, payer, leafIndex, originalCompressedNFT),
          leafIndex: 0,
          proof: [],
        }
      );
      const tx = new Transaction().add(verifyLeafIx);
      const txId = await sendAndConfirmTransaction(connection, tx, [payerKeypair], {
        commitment: 'confirmed',
        skipPreflight: true,
      });
      console.log('Verified NFT existence:', txId);
    });

    it('Can transfer and burn a compressed NFT', async () => {
      // Transfer.
      const accountInfo = await connection.getAccountInfo(merkleTree, { commitment: 'confirmed' });
      const account = ConcurrentMerkleTreeAccount.fromBuffer(accountInfo!.data!);
      const [treeAuthority] = PublicKey.findProgramAddressSync(
        [merkleTree.toBuffer()],
        BUBBLEGUM_PROGRAM_ID,
      );
      const newLeafOwnerKeypair = new Keypair();
      const newLeafOwner = newLeafOwnerKeypair.publicKey;

      const transferIx = createTransferInstruction(
        {
          treeAuthority,
          leafOwner: payer,
          leafDelegate: payer,
          newLeafOwner,
          merkleTree,
          logWrapper: SPL_NOOP_PROGRAM_ID,
          compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        },
        {
          root: Array.from(account.getCurrentRoot()),
          dataHash: Array.from(computeDataHash(originalCompressedNFT)),
          creatorHash: Array.from(computeCreatorHash(originalCompressedNFT.creators)),
          nonce: 0,
          index: 0
        },
      );

      const transferTx = new Transaction().add(transferIx);
      transferTx.feePayer = payer;
      const transferTxId = await sendAndConfirmTransaction(connection, transferTx, [payerKeypair], {
        commitment: 'confirmed',
        skipPreflight: true,
      });

      console.log('NFT transfer tx:', transferTxId);

      // Burn.
      const burnIx = createBurnInstruction(
        {
          treeAuthority,
          leafOwner: newLeafOwner,
          leafDelegate: newLeafOwner,
          merkleTree,
          logWrapper: SPL_NOOP_PROGRAM_ID,
          compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        },
        {
          root: Array.from(account.getCurrentRoot()),
          dataHash: Array.from(computeDataHash(originalCompressedNFT)),
          creatorHash: Array.from(computeCreatorHash(originalCompressedNFT.creators)),
          nonce: 0,
          index: 0
        },
      );

      const burnTx = new Transaction().add(burnIx);
      burnTx.feePayer = payer;
      const burnTxId = await sendAndConfirmTransaction(connection, burnTx, [payerKeypair, newLeafOwnerKeypair], {
        commitment: 'confirmed',
        skipPreflight: true,
      });

      console.log('NFT burn tx:', burnTxId);
    });

    it('Can redeem and decompress compressed NFT', async () => {
      // Redeem.
      const accountInfo = await connection.getAccountInfo(merkleTree, { commitment: 'confirmed' });
      const account = ConcurrentMerkleTreeAccount.fromBuffer(accountInfo!.data!);
      const [treeAuthority] = PublicKey.findProgramAddressSync(
        [merkleTree.toBuffer()],
        BUBBLEGUM_PROGRAM_ID,
      );
      const nonce = new BN.BN(0);
      const [voucher] = PublicKey.findProgramAddressSync(
        [Buffer.from('voucher', 'utf8'), merkleTree.toBuffer(), Uint8Array.from(nonce.toArray('le', 8))],
        BUBBLEGUM_PROGRAM_ID,
      );

      const redeemIx = createRedeemInstruction(
        {
          treeAuthority,
          leafOwner: payer,
          leafDelegate: payer,
          merkleTree,
          voucher,
          logWrapper: SPL_NOOP_PROGRAM_ID,
          compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        },
        {
          root: Array.from(account.getCurrentRoot()),
          dataHash: Array.from(computeDataHash(originalCompressedNFT)),
          creatorHash: Array.from(computeCreatorHash(originalCompressedNFT.creators)),
          nonce,
          index: 0
        },
      );

      const redeemTx = new Transaction().add(redeemIx);
      redeemTx.feePayer = payer;
      const redeemTxId = await sendAndConfirmTransaction(connection, redeemTx, [payerKeypair], {
        commitment: 'confirmed',
        skipPreflight: true,
      });

      console.log('NFT redeem tx:', redeemTxId);

      // Decompress.
      const [mint] = PublicKey.findProgramAddressSync(
        [Buffer.from('asset', 'utf8'), merkleTree.toBuffer(), Uint8Array.from(nonce.toArray('le', 8))],
        BUBBLEGUM_PROGRAM_ID,
      );
      const [tokenAccount] = PublicKey.findProgramAddressSync(
        [payer.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID,
      );
      const [mintAuthority] = PublicKey.findProgramAddressSync(
        [mint.toBuffer()],
        BUBBLEGUM_PROGRAM_ID
      );
      const [metadata] = PublicKey.findProgramAddressSync(
        [Buffer.from('metadata', 'utf8'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        TOKEN_METADATA_PROGRAM_ID,
      );
      const [masterEdition] = PublicKey.findProgramAddressSync(
        [Buffer.from('metadata', 'utf8'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition', 'utf8')],
        TOKEN_METADATA_PROGRAM_ID,
      );

      const decompressIx = createDecompressV1Instruction(
        {
          voucher,
          leafOwner: payer,
          tokenAccount,
          mint,
          mintAuthority,
          metadata,
          masterEdition,
          sysvarRent: SYSVAR_RENT_PUBKEY,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          logWrapper: SPL_NOOP_PROGRAM_ID,
        },
        {
          metadata: originalCompressedNFT
        },
      );

      const decompressTx = new Transaction().add(decompressIx);
      decompressTx.feePayer = payer;
      const decompressTxId = await sendAndConfirmTransaction(connection, decompressTx, [payerKeypair], {
        commitment: 'confirmed',
        skipPreflight: true,
      });

      console.log('NFT decompress tx:', decompressTxId);
    });

    // TODO(@metaplex): add collection tests here
  });
});
