import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const ApproveCollectionAuthorityStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'ApproveCollectionAuthorityInstructionArgs',
);
export type ApproveCollectionAuthorityInstructionAccounts = {
  collectionAuthorityRecord: web3.PublicKey;
  newCollectionAuthority: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  payer: web3.PublicKey;
  metadata: web3.PublicKey;
  mint: web3.PublicKey;
  systemAccount: web3.PublicKey;
};

const approveCollectionAuthorityInstructionDiscriminator = [254, 136, 208, 39, 65, 66, 27, 111];

/**
 * Creates a _ApproveCollectionAuthority_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createApproveCollectionAuthorityInstruction(
  accounts: ApproveCollectionAuthorityInstructionAccounts,
) {
  const {
    collectionAuthorityRecord,
    newCollectionAuthority,
    updateAuthority,
    payer,
    metadata,
    mint,
    systemAccount,
  } = accounts;

  const [data] = ApproveCollectionAuthorityStruct.serialize({
    instructionDiscriminator: approveCollectionAuthorityInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: collectionAuthorityRecord,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: newCollectionAuthority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: updateAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: payer,
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
    {
      pubkey: systemAccount,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: web3.SYSVAR_RENT_PUBKEY,
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
