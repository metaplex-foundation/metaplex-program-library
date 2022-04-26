/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import {
  MintNewEditionFromMasterEditionViaTokenArgs,
  mintNewEditionFromMasterEditionViaTokenArgsBeet,
} from '../types/MintNewEditionFromMasterEditionViaTokenArgs';

/**
 * @category Instructions
 * @category MintNewEditionFromMasterEditionViaVaultProxy
 * @category generated
 */
export type MintNewEditionFromMasterEditionViaVaultProxyInstructionArgs = {
  mintNewEditionFromMasterEditionViaTokenArgs: MintNewEditionFromMasterEditionViaTokenArgs;
};
/**
 * @category Instructions
 * @category MintNewEditionFromMasterEditionViaVaultProxy
 * @category generated
 */
const MintNewEditionFromMasterEditionViaVaultProxyStruct = new beet.BeetArgsStruct<
  MintNewEditionFromMasterEditionViaVaultProxyInstructionArgs & {
    instructionDiscriminator: number;
  }
>(
  [
    ['instructionDiscriminator', beet.u8],
    [
      'mintNewEditionFromMasterEditionViaTokenArgs',
      mintNewEditionFromMasterEditionViaTokenArgsBeet,
    ],
  ],
  'MintNewEditionFromMasterEditionViaVaultProxyInstructionArgs',
);
/**
 * Accounts required by the _MintNewEditionFromMasterEditionViaVaultProxy_ instruction
 *
 * @property [_writable_] newMetadata New Metadata key (pda of ['metadata', program id, mint id])
 * @property [_writable_] newEdition New Edition (pda of ['metadata', program id, mint id, 'edition'])
 * @property [_writable_] masterEdition Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition']
 * @property [_writable_] newMint Mint of new token - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
 * @property [_writable_] editionMarkPda Edition pda to mark creation - will be checked for pre-existence. (pda of ['metadata', program id, master metadata mint id, 'edition', edition_number]) where edition_number is NOT the edition number you pass in args but actually edition_number = floor(edition/EDITION_MARKER_BIT_SIZE).
 * @property [**signer**] newMintAuthority Mint authority of new mint
 * @property [_writable_, **signer**] payer payer
 * @property [**signer**] vaultAuthority Vault authority
 * @property [] safetyDepositStore Safety deposit token store account
 * @property [] safetyDepositBox Safety deposit box
 * @property [] vault Vault
 * @property [] newMetadataUpdateAuthority Update authority info for new metadata
 * @property [] metadata Master record metadata account
 * @property [] tokenVaultProgram Token vault program
 * @category Instructions
 * @category MintNewEditionFromMasterEditionViaVaultProxy
 * @category generated
 */
export type MintNewEditionFromMasterEditionViaVaultProxyInstructionAccounts = {
  newMetadata: web3.PublicKey;
  newEdition: web3.PublicKey;
  masterEdition: web3.PublicKey;
  newMint: web3.PublicKey;
  editionMarkPda: web3.PublicKey;
  newMintAuthority: web3.PublicKey;
  payer: web3.PublicKey;
  vaultAuthority: web3.PublicKey;
  safetyDepositStore: web3.PublicKey;
  safetyDepositBox: web3.PublicKey;
  vault: web3.PublicKey;
  newMetadataUpdateAuthority: web3.PublicKey;
  metadata: web3.PublicKey;
  tokenVaultProgram: web3.PublicKey;
};

const mintNewEditionFromMasterEditionViaVaultProxyInstructionDiscriminator = 13;

/**
 * Creates a _MintNewEditionFromMasterEditionViaVaultProxy_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category MintNewEditionFromMasterEditionViaVaultProxy
 * @category generated
 */
export function createMintNewEditionFromMasterEditionViaVaultProxyInstruction(
  accounts: MintNewEditionFromMasterEditionViaVaultProxyInstructionAccounts,
  args: MintNewEditionFromMasterEditionViaVaultProxyInstructionArgs,
) {
  const {
    newMetadata,
    newEdition,
    masterEdition,
    newMint,
    editionMarkPda,
    newMintAuthority,
    payer,
    vaultAuthority,
    safetyDepositStore,
    safetyDepositBox,
    vault,
    newMetadataUpdateAuthority,
    metadata,
    tokenVaultProgram,
  } = accounts;

  const [data] = MintNewEditionFromMasterEditionViaVaultProxyStruct.serialize({
    instructionDiscriminator: mintNewEditionFromMasterEditionViaVaultProxyInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: newMetadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: newEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: masterEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: newMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: editionMarkPda,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: newMintAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: payer,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: vaultAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: safetyDepositStore,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: safetyDepositBox,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: vault,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: newMetadataUpdateAuthority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: tokenVaultProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: web3.SystemProgram.programId,
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
