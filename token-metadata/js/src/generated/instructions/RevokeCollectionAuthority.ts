import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const RevokeCollectionAuthorityStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'RevokeCollectionAuthorityInstructionArgs',
);
export type RevokeCollectionAuthorityInstructionAccounts = {
  collectionAuthorityRecord: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  metadata: web3.PublicKey;
  mint: web3.PublicKey;
};

const revokeCollectionAuthorityInstructionDiscriminator = [31, 139, 135, 198, 29, 48, 160, 154];

/**
 * Creates a _RevokeCollectionAuthority_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createRevokeCollectionAuthorityInstruction(
  accounts: RevokeCollectionAuthorityInstructionAccounts,
) {
  const { collectionAuthorityRecord, updateAuthority, metadata, mint } = accounts;

  const [data] = RevokeCollectionAuthorityStruct.serialize({
    instructionDiscriminator: revokeCollectionAuthorityInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: collectionAuthorityRecord,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: updateAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: mint,
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
