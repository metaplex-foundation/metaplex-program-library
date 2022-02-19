import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const ConvertMasterEditionV1ToV2Struct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'ConvertMasterEditionV1ToV2InstructionArgs',
);
export type ConvertMasterEditionV1ToV2InstructionAccounts = {
  masterEdition: web3.PublicKey;
  oneTimeAuth: web3.PublicKey;
  printingMint: web3.PublicKey;
};

const convertMasterEditionV1ToV2InstructionDiscriminator = [217, 26, 108, 0, 55, 126, 167, 238];

/**
 * Creates a _ConvertMasterEditionV1ToV2_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createConvertMasterEditionV1ToV2Instruction(
  accounts: ConvertMasterEditionV1ToV2InstructionAccounts,
) {
  const { masterEdition, oneTimeAuth, printingMint } = accounts;

  const [data] = ConvertMasterEditionV1ToV2Struct.serialize({
    instructionDiscriminator: convertMasterEditionV1ToV2InstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: masterEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: oneTimeAuth,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: printingMint,
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
