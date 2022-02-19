import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const PuffMetadataStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'PuffMetadataInstructionArgs',
);
export type PuffMetadataInstructionAccounts = {
  metadata: web3.PublicKey;
};

const puffMetadataInstructionDiscriminator = [87, 217, 21, 132, 105, 238, 71, 114];

/**
 * Creates a _PuffMetadata_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createPuffMetadataInstruction(accounts: PuffMetadataInstructionAccounts) {
  const { metadata } = accounts;

  const [data] = PuffMetadataStruct.serialize({
    instructionDiscriminator: puffMetadataInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: metadata,
      isWritable: true,
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
