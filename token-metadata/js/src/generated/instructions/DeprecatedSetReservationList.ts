import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type DeprecatedSetReservationListInstructionArgs = {
  setReservationListArgs: definedTypes.SetReservationListArgs;
};
const DeprecatedSetReservationListStruct = new beet.FixableBeetArgsStruct<
  DeprecatedSetReservationListInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['setReservationListArgs', definedTypes.setReservationListArgsStruct],
  ],
  'DeprecatedSetReservationListInstructionArgs',
);
export type DeprecatedSetReservationListInstructionAccounts = {
  masterEdition: web3.PublicKey;
  reservationList: web3.PublicKey;
  resource: web3.PublicKey;
};

const deprecatedSetReservationListInstructionDiscriminator = [68, 28, 66, 19, 59, 203, 190, 142];

/**
 * Creates a _DeprecatedSetReservationList_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createDeprecatedSetReservationListInstruction(
  accounts: DeprecatedSetReservationListInstructionAccounts,
  args: DeprecatedSetReservationListInstructionArgs,
) {
  const { masterEdition, reservationList, resource } = accounts;

  const [data] = DeprecatedSetReservationListStruct.serialize({
    instructionDiscriminator: deprecatedSetReservationListInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: masterEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: reservationList,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: resource,
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
