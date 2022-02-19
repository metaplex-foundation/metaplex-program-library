import * as splToken from '@solana/spl-token';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const ThawDelegatedAccountStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'ThawDelegatedAccountInstructionArgs',
);
export type ThawDelegatedAccountInstructionAccounts = {
  delegate: web3.PublicKey;
  tokenAccount: web3.PublicKey;
  edition: web3.PublicKey;
  mint: web3.PublicKey;
};

const thawDelegatedAccountInstructionDiscriminator = [239, 152, 227, 34, 225, 200, 206, 170];

/**
 * Creates a _ThawDelegatedAccount_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createThawDelegatedAccountInstruction(
  accounts: ThawDelegatedAccountInstructionAccounts,
) {
  const { delegate, tokenAccount, edition, mint } = accounts;

  const [data] = ThawDelegatedAccountStruct.serialize({
    instructionDiscriminator: thawDelegatedAccountInstructionDiscriminator,
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
