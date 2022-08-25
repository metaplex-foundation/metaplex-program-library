/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * @category Instructions
 * @category AddVoucherToPack
 * @category generated
 */
export const AddVoucherToPackStruct = new beet.BeetArgsStruct<{ instructionDiscriminator: number }>(
  [['instructionDiscriminator', beet.u8]],
  'AddVoucherToPackInstructionArgs',
);
/**
 * Accounts required by the _AddVoucherToPack_ instruction
 *
 * @property [_writable_] packSet
 * @property [_writable_] packVoucher PDA, ['voucher', pack, index]
 * @property [_writable_, **signer**] authority
 * @property [**signer**] voucherOwner
 * @property [] masterEdition
 * @property [] masterMetadata
 * @property [] mint
 * @property [_writable_] source
 * @property [] store
 * @category Instructions
 * @category AddVoucherToPack
 * @category generated
 */
export type AddVoucherToPackInstructionAccounts = {
  packSet: web3.PublicKey;
  packVoucher: web3.PublicKey;
  authority: web3.PublicKey;
  voucherOwner: web3.PublicKey;
  masterEdition: web3.PublicKey;
  masterMetadata: web3.PublicKey;
  mint: web3.PublicKey;
  source: web3.PublicKey;
  store: web3.PublicKey;
  rent?: web3.PublicKey;
  systemProgram?: web3.PublicKey;
  tokenProgram?: web3.PublicKey;
};

export const addVoucherToPackInstructionDiscriminator = 2;

/**
 * Creates a _AddVoucherToPack_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category AddVoucherToPack
 * @category generated
 */
export function createAddVoucherToPackInstruction(
  accounts: AddVoucherToPackInstructionAccounts,
  programId = new web3.PublicKey('packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu'),
) {
  const [data] = AddVoucherToPackStruct.serialize({
    instructionDiscriminator: addVoucherToPackInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.packSet,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.packVoucher,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.authority,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.voucherOwner,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.masterEdition,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.masterMetadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.mint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.source,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.store,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.rent ?? web3.SYSVAR_RENT_PUBKEY,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenProgram ?? splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}
