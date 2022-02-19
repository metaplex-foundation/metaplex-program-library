import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const DeprecatedCreateReservationListStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'DeprecatedCreateReservationListInstructionArgs',
);
export type DeprecatedCreateReservationListInstructionAccounts = {
  reservationList: web3.PublicKey;
  payer: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  masterEdition: web3.PublicKey;
  resource: web3.PublicKey;
  metadata: web3.PublicKey;
  systemAccount: web3.PublicKey;
};

const deprecatedCreateReservationListInstructionDiscriminator = [
  171, 227, 161, 158, 1, 176, 105, 72,
];

/**
 * Creates a _DeprecatedCreateReservationList_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createDeprecatedCreateReservationListInstruction(
  accounts: DeprecatedCreateReservationListInstructionAccounts,
) {
  const {
    reservationList,
    payer,
    updateAuthority,
    masterEdition,
    resource,
    metadata,
    systemAccount,
  } = accounts;

  const [data] = DeprecatedCreateReservationListStruct.serialize({
    instructionDiscriminator: deprecatedCreateReservationListInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: reservationList,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: payer,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: updateAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: masterEdition,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: resource,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: metadata,
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
