import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from '@solana/web3.js';

import {
  getConcurrentMerkleTreeAccountSize,
  createVerifyLeafIx,
  ConcurrentMerkleTreeAccount,
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  SPL_NOOP_PROGRAM_ID,
  ValidDepthSizePair,
  createVerifyLeafInstruction,
} from '@solana/spl-account-compression';

import {
  createCreateTreeInstruction,
  createMintV1Instruction,
  MetadataArgs,
  PROGRAM_ID as BUBBLEGUM_PROGRAM_ID,
  TokenProgramVersion,
  TokenStandard,
  Creator,
} from '../src/generated';
import { getLeafAssetId, computeCompressedNFTHash } from '../src/mpl-bubblegum';
import { BN } from 'bn.js';

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
  const payerKeypair = keypairFromSeed('metaplex-test');
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
    const originalCompressedNFT = makeCompressedNFT('test', 'TST');
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

      // Verify leaf exists
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

    // TODO(@metaplex): add collection tests here
  });
});
