import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const RemoveCreatorVerificationStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'RemoveCreatorVerificationInstructionArgs',
);
export type RemoveCreatorVerificationInstructionAccounts = {
  metadata: web3.PublicKey;
  creator: web3.PublicKey;
};

const removeCreatorVerificationInstructionDiscriminator = [41, 194, 140, 217, 90, 160, 139, 6];

/**
 * Creates a _RemoveCreatorVerification_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createRemoveCreatorVerificationInstruction(
  accounts: RemoveCreatorVerificationInstructionAccounts,
) {
  const { metadata, creator } = accounts;

  const [data] = RemoveCreatorVerificationStruct.serialize({
    instructionDiscriminator: removeCreatorVerificationInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: creator,
      isWritable: false,
      isSigner: true,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
    keys,
    data,
  });
  return ix;
}
