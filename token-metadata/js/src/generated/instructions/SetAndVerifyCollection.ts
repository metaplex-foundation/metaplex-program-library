import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const SetAndVerifyCollectionStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'SetAndVerifyCollectionInstructionArgs',
);
export type SetAndVerifyCollectionInstructionAccounts = {
  metadata: web3.PublicKey;
  collectionAuthority: web3.PublicKey;
  payer: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  collectionMint: web3.PublicKey;
  collection: web3.PublicKey;
  collectionMasterEditionAccount: web3.PublicKey;
  collectionAuthorityRecord: web3.PublicKey;
};

const setAndVerifyCollectionInstructionDiscriminator = [235, 242, 121, 216, 158, 234, 180, 234];

/**
 * Creates a _SetAndVerifyCollection_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createSetAndVerifyCollectionInstruction(
  accounts: SetAndVerifyCollectionInstructionAccounts,
) {
  const {
    metadata,
    collectionAuthority,
    payer,
    updateAuthority,
    collectionMint,
    collection,
    collectionMasterEditionAccount,
    collectionAuthorityRecord,
  } = accounts;

  const [data] = SetAndVerifyCollectionStruct.serialize({
    instructionDiscriminator: setAndVerifyCollectionInstructionDiscriminator,
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
      pubkey: payer,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: updateAuthority,
      isWritable: false,
      isSigner: false,
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
