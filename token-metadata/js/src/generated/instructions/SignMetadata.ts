import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const SignMetadataStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'SignMetadataInstructionArgs',
);
export type SignMetadataInstructionAccounts = {
  metadata: web3.PublicKey;
  creator: web3.PublicKey;
};

const signMetadataInstructionDiscriminator = [178, 245, 253, 205, 236, 250, 233, 209];

/**
 * Creates a _SignMetadata_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createSignMetadataInstruction(accounts: SignMetadataInstructionAccounts) {
  const { metadata, creator } = accounts;

  const [data] = SignMetadataStruct.serialize({
    instructionDiscriminator: signMetadataInstructionDiscriminator,
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
