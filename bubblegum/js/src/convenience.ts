import { BN } from "@project-serum/anchor";
import {
  TransactionInstruction,
  PublicKey,
  Connection,
  AccountInfo,
} from "@solana/web3.js";
import { keccak_256 } from "js-sha3";
import { Creator, TreeConfig, MintRequest, PROGRAM_ID } from "./generated";
import {
  CANDY_WRAPPER_PROGRAM_ID,
  bufferToArray,
  num16ToBuffer,
} from "../../utils";
import {
  PROGRAM_ID as GUMMYROLL_PROGRAM_ID,
  createAllocTreeIx,
} from "../../gummyroll";
import { createCreateTreeInstruction } from "./generated";
import { assert } from "chai";

export async function getBubblegumAuthorityPDA(merkleRollPubKey: PublicKey) {
  const [bubblegumAuthorityPDAKey] = await PublicKey.findProgramAddress(
    [merkleRollPubKey.toBuffer()],
    PROGRAM_ID
  );
  return bubblegumAuthorityPDAKey;
}

export async function getDefaultMintRequestPDA(merkleRollPubKey: PublicKey) {
  const authority = await getBubblegumAuthorityPDA(merkleRollPubKey);
  const [defaultMintRequestKey] = await PublicKey.findProgramAddress(
    [merkleRollPubKey.toBuffer(), authority.toBuffer()],
    PROGRAM_ID
  );
  return defaultMintRequestKey;
}

export async function getMintRequestPDA(
  merkleRollPubKey: PublicKey,
  requester: PublicKey
) {
  const [mintAuthorityRequest] = await PublicKey.findProgramAddress(
    [merkleRollPubKey.toBuffer(), requester.toBuffer()],
    PROGRAM_ID
  );
  return mintAuthorityRequest;
}

export async function getMintRequest(
  connection: Connection,
  merkleRollPubKey: PublicKey,
  requester: PublicKey
): Promise<MintRequest> {
  const requestPda = await getMintRequestPDA(merkleRollPubKey, requester);
  return await MintRequest.fromAccountAddress(connection, requestPda);
}

export async function assertOnChainMintRequest(
  connection: Connection,
  expectedState: MintRequest,
  mintRequestPDA: PublicKey
) {
  const request = await MintRequest.fromAccountAddress(
    connection,
    mintRequestPDA
  );
  const { mintAuthority, numMintsApproved, numMintsRequested } = expectedState;
  assert(
    request.mintAuthority.equals(mintAuthority),
    `Request should have mint authority ${mintAuthority.toString()}, but has ${request.mintAuthority.toString()}`
  );
  assert(
    new BN(request.numMintsApproved).eq(new BN(numMintsApproved)),
    `Request should${numMintsApproved > 0 ? "" : " not"} be approved`
  );
  assert(
    new BN(request.numMintsRequested).eq(new BN(numMintsRequested)),
    `Request should have requested ${numMintsRequested}, but has ${request.numMintsRequested}`
  );
}

export async function assertOnChainTreeAuthority(
  connection: Connection,
  expectedState: TreeConfig,
  authorityPDA: PublicKey
) {
  const authority = await TreeConfig.fromAccountAddress(
    connection,
    authorityPDA
  );
  const { creator, delegate, totalMintCapacity, numMintsApproved, numMinted } =
    expectedState;
  assert(
    authority.creator.equals(creator),
    `Authority should have creator ${creator.toString()}, but has ${authority.creator.toString()}`
  );
  assert(
    authority.delegate.equals(delegate),
    `Authority should have delegate ${delegate.toString()}, but has ${authority.delegate.toString()}`
  );
  assert(
    new BN(authority.totalMintCapacity).eq(new BN(totalMintCapacity)),
    `Authority should have total mint capacity ${totalMintCapacity}, but has ${authority.totalMintCapacity}`
  );
  assert(
    new BN(authority.numMintsApproved).eq(new BN(numMintsApproved)),
    `Authority should have num mints approved ${numMintsApproved}, but has ${authority.numMintsApproved}`
  );
  assert(
    new BN(authority.numMinted).eq(new BN(numMinted)),
    `Authority should have num minted ${numMinted}, but has ${authority.numMinted}`
  );
}

export async function getNonceCount(
  connection: Connection,
  tree: PublicKey
): Promise<BN> {
  const treeAuthority = await getBubblegumAuthorityPDA(tree);
  return new BN(
    (await TreeConfig.fromAccountAddress(connection, treeAuthority)).numMinted
  );
}

export async function getVoucherPDA(
  connection: Connection,
  tree: PublicKey,
  leafIndex: number
): Promise<PublicKey> {
  let [voucher] = await PublicKey.findProgramAddress(
    [
      Buffer.from("voucher", "utf8"),
      tree.toBuffer(),
      Uint8Array.from((new BN(leafIndex)).toArray("le", 8)),
    ],
    PROGRAM_ID
  );
  return voucher;
}

export async function getLeafAssetId(
  tree: PublicKey,
  leafIndex: BN
): Promise<PublicKey> {
  let [assetId] = await PublicKey.findProgramAddress(
    [
      Buffer.from("asset", "utf8"),
      tree.toBuffer(),
      Uint8Array.from(leafIndex.toArray("le", 8)),
    ],
    PROGRAM_ID
  );
  return assetId;
}

export async function getCreateTreeIxs(
  connection: Connection,
  maxDepth: number,
  maxBufferSize: number,
  canopyDepth: number,
  payer: PublicKey,
  merkleRoll: PublicKey,
  treeCreator: PublicKey
): Promise<TransactionInstruction[]> {
  const allocAccountIx = await createAllocTreeIx(
    connection,
    maxBufferSize,
    maxDepth,
    canopyDepth,
    payer,
    merkleRoll
  );
  const authority = await getBubblegumAuthorityPDA(merkleRoll);
  const initGummyrollIx = createCreateTreeInstruction(
    {
      treeCreator,
      payer,
      authority,
      candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
      gummyrollProgram: GUMMYROLL_PROGRAM_ID,
      merkleSlab: merkleRoll,
    },
    {
      maxDepth,
      maxBufferSize,
    }
  );
  return [allocAccountIx, initGummyrollIx];
}

export function computeMetadataArgsHash(mintIx: TransactionInstruction) {
  const metadataArgsBuffer = mintIx.data.slice(8);
  return keccak_256.digest(metadataArgsBuffer);
}

export function computeDataHash(
  sellerFeeBasisPoints: number,
  mintIx?: TransactionInstruction,
  metadataArgsHash?: number[]
) {
  // Input validation
  if (
    typeof mintIx === "undefined" &&
    typeof metadataArgsHash === "undefined"
  ) {
    throw new Error(
      "Either the mint NFT instruction or the hash of metadata args must be provided to determine the data hash of the leaf!"
    );
  }
  if (
    typeof mintIx !== "undefined" &&
    typeof metadataArgsHash !== "undefined"
  ) {
    throw new Error(
      "Only the mint instruction or the hash of metadata args should be specified, not both"
    );
  }

  if (typeof mintIx !== "undefined") {
    metadataArgsHash = computeMetadataArgsHash(mintIx);
  }

  const sellerFeeBasisPointsNumberArray = bufferToArray(
    num16ToBuffer(sellerFeeBasisPoints)
  );
  const allDataToHash = metadataArgsHash!.concat(
    sellerFeeBasisPointsNumberArray
  );
  const dataHashOfCompressedNFT = bufferToArray(
    Buffer.from(keccak_256.digest(allDataToHash))
  );
  return dataHashOfCompressedNFT;
}

export function computeCreatorHash(creators: Creator[]) {
  let bufferOfCreatorData = Buffer.from([]);
  let bufferOfCreatorShares = Buffer.from([]);
  for (let creator of creators) {
    bufferOfCreatorData = Buffer.concat([
      bufferOfCreatorData,
      creator.address.toBuffer(),
      Buffer.from([creator.share]),
    ]);
    bufferOfCreatorShares = Buffer.concat([
      bufferOfCreatorShares,
      Buffer.from([creator.share]),
    ]);
  }
  let creatorHash = bufferToArray(
    Buffer.from(keccak_256.digest(bufferOfCreatorData))
  );
  return creatorHash;
}
