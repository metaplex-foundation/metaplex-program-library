import * as splToken from '@solana/spl-token';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const FreezeDelegatedAccountStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'FreezeDelegatedAccountInstructionArgs',
);
export type FreezeDelegatedAccountInstructionAccounts = {
  delegate: web3.PublicKey;
  tokenAccount: web3.PublicKey;
  edition: web3.PublicKey;
  mint: web3.PublicKey;
};

const freezeDelegatedAccountInstructionDiscriminator = [14, 16, 189, 180, 116, 19, 96, 127];

/**
 * Creates a _FreezeDelegatedAccount_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createFreezeDelegatedAccountInstruction(
  accounts: FreezeDelegatedAccountInstructionAccounts,
) {
  const { delegate, tokenAccount, edition, mint } = accounts;

  const [data] = FreezeDelegatedAccountStruct.serialize({
    instructionDiscriminator: freezeDelegatedAccountInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: delegate,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: tokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: edition,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: mint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: splToken.TOKEN_PROGRAM_ID,
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
