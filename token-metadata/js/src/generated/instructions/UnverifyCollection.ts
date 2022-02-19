import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const UnverifyCollectionStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'UnverifyCollectionInstructionArgs',
);
export type UnverifyCollectionInstructionAccounts = {
  metadata: web3.PublicKey;
  collectionAuthority: web3.PublicKey;
  collectionMint: web3.PublicKey;
  collection: web3.PublicKey;
  collectionMasterEditionAccount: web3.PublicKey;
  collectionAuthorityRecord: web3.PublicKey;
};

const unverifyCollectionInstructionDiscriminator = [250, 251, 42, 106, 41, 137, 186, 168];

/**
 * Creates a _UnverifyCollection_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createUnverifyCollectionInstruction(
  accounts: UnverifyCollectionInstructionAccounts,
) {
  const {
    metadata,
    collectionAuthority,
    collectionMint,
    collection,
    collectionMasterEditionAccount,
    collectionAuthorityRecord,
  } = accounts;

  const [data] = UnverifyCollectionStruct.serialize({
    instructionDiscriminator: unverifyCollectionInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: collectionAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: collectionMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: collection,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: collectionMasterEditionAccount,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: collectionAuthorityRecord,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
    keys,
    data,
  });
  return ix;
}
