import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const UpdatePrimarySaleHappenedViaTokenStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'UpdatePrimarySaleHappenedViaTokenInstructionArgs',
);
export type UpdatePrimarySaleHappenedViaTokenInstructionAccounts = {
  metadata: web3.PublicKey;
  owner: web3.PublicKey;
  token: web3.PublicKey;
};

const updatePrimarySaleHappenedViaTokenInstructionDiscriminator = [
  172, 129, 173, 210, 222, 129, 243, 98,
];

/**
 * Creates a _UpdatePrimarySaleHappenedViaToken_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createUpdatePrimarySaleHappenedViaTokenInstruction(
  accounts: UpdatePrimarySaleHappenedViaTokenInstructionAccounts,
) {
  const { metadata, owner, token } = accounts;

  const [data] = UpdatePrimarySaleHappenedViaTokenStruct.serialize({
    instructionDiscriminator: updatePrimarySaleHappenedViaTokenInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: owner,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: token,
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
