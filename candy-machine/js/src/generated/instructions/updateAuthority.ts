import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

export type UpdateAuthorityInstructionArgs = {
  newAuthority: beet.COption<web3.PublicKey>;
};
const updateAuthorityStruct = new beet.FixableBeetArgsStruct<
  UpdateAuthorityInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['newAuthority', beet.coption(beetSolana.publicKey)],
  ],
  'UpdateAuthorityInstructionArgs',
);
/**
 * Accounts required by the _updateAuthority_ instruction
 */
export type UpdateAuthorityInstructionAccounts = {
  candyMachine: web3.PublicKey;
  authority: web3.PublicKey;
  wallet: web3.PublicKey;
};

const updateAuthorityInstructionDiscriminator = [32, 46, 64, 28, 149, 75, 243, 88];

/**
 * Creates a _UpdateAuthority_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createUpdateAuthorityInstruction(
  accounts: UpdateAuthorityInstructionAccounts,
  args: UpdateAuthorityInstructionArgs,
) {
  const { candyMachine, authority, wallet } = accounts;

  const [data] = updateAuthorityStruct.serialize({
    instructionDiscriminator: updateAuthorityInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: candyMachine,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: authority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: wallet,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ'),
    keys,
    data,
  });
  return ix;
}
