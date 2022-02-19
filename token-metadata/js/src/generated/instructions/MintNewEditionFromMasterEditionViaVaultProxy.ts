import * as splToken from '@solana/spl-token';
import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type MintNewEditionFromMasterEditionViaVaultProxyInstructionArgs = {
  mintNewEditionFromMasterEditionViaTokenArgs: definedTypes.MintNewEditionFromMasterEditionViaTokenArgs;
};
const MintNewEditionFromMasterEditionViaVaultProxyStruct = new beet.BeetArgsStruct<
  MintNewEditionFromMasterEditionViaVaultProxyInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    [
      'mintNewEditionFromMasterEditionViaTokenArgs',
      definedTypes.mintNewEditionFromMasterEditionViaTokenArgsStruct,
    ],
  ],
  'MintNewEditionFromMasterEditionViaVaultProxyInstructionArgs',
);
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
  systemAccount: web3.PublicKey;
};

const mintNewEditionFromMasterEditionViaVaultProxyInstructionDiscriminator = [
  66, 246, 206, 73, 249, 35, 194, 47,
];

/**
 * Creates a _MintNewEditionFromMasterEditionViaVaultProxy_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
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
    systemAccount,
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
      isWritable: false,
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
