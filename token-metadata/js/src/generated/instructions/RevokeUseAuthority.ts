import * as splToken from '@solana/spl-token';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const RevokeUseAuthorityStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'RevokeUseAuthorityInstructionArgs',
);
export type RevokeUseAuthorityInstructionAccounts = {
  useAuthorityRecord: web3.PublicKey;
  owner: web3.PublicKey;
  user: web3.PublicKey;
  ownerTokenAccount: web3.PublicKey;
  mint: web3.PublicKey;
  metadata: web3.PublicKey;
  systemAccount: web3.PublicKey;
};

const revokeUseAuthorityInstructionDiscriminator = [204, 194, 208, 141, 142, 221, 109, 84];

/**
 * Creates a _RevokeUseAuthority_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createRevokeUseAuthorityInstruction(
  accounts: RevokeUseAuthorityInstructionAccounts,
) {
  const { useAuthorityRecord, owner, user, ownerTokenAccount, mint, metadata, systemAccount } =
    accounts;

  const [data] = RevokeUseAuthorityStruct.serialize({
    instructionDiscriminator: revokeUseAuthorityInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: useAuthorityRecord,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: owner,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: user,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: ownerTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: mint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: splToken.TOKEN_PROGRAM_ID,
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
