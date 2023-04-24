import { PROGRAM_ID, Creator, MetadataArgs, metadataArgsBeet } from './generated';
import { keccak_256 } from 'js-sha3';
import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

export * from './generated';

export async function getLeafAssetId(tree: PublicKey, leafIndex: BN): Promise<PublicKey> {
  const [assetId] = await PublicKey.findProgramAddress(
    [Buffer.from('asset', 'utf8'), tree.toBuffer(), Uint8Array.from(leafIndex.toArray('le', 8))],
    PROGRAM_ID,
  );
  return assetId;
}

export function computeDataHash(metadata: MetadataArgs): Buffer {
  const [serializedMetadata] = metadataArgsBeet.serialize(metadata);
  const metadataHash = Buffer.from(keccak_256.digest(serializedMetadata));

  const sellerFeeBasisPointsBuffer = new BN(metadata.sellerFeeBasisPoints).toBuffer('le', 2);

  return Buffer.from(keccak_256.digest(Buffer.concat([metadataHash, sellerFeeBasisPointsBuffer])));
}

export function computeCreatorHash(creators: Creator[]): Buffer {
  let bufferOfCreatorData = Buffer.from([]);
  for (const creator of creators) {
    bufferOfCreatorData = Buffer.concat([
      bufferOfCreatorData,
      creator.address.toBuffer(),
      Buffer.from([creator.verified ? 1 : 0]),
      Buffer.from([creator.share]),
    ]);
  }
  return Buffer.from(keccak_256.digest(bufferOfCreatorData));
}

export function computeCompressedNFTHash(
  assetId: PublicKey,
  owner: PublicKey,
  delegate: PublicKey,
  treeNonce: BN,
  metadata: MetadataArgs,
): Buffer {
  const message = Buffer.concat([
    Buffer.from([0x1]), // All NFTs are version 1 right now
    assetId.toBuffer(),
    owner.toBuffer(),
    delegate.toBuffer(),
    treeNonce.toBuffer('le', 8),
    computeDataHash(metadata),
    computeCreatorHash(metadata.creators),
  ]);

  return Buffer.from(keccak_256.digest(message));
}
