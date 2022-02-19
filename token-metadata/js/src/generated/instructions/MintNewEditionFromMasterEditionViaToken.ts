import * as splToken from '@solana/spl-token';
import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type MintNewEditionFromMasterEditionViaTokenInstructionArgs = {
  mintNewEditionFromMasterEditionViaTokenArgs: definedTypes.MintNewEditionFromMasterEditionViaTokenArgs;
};
const MintNewEditionFromMasterEditionViaTokenStruct = new beet.BeetArgsStruct<
  MintNewEditionFromMasterEditionViaTokenInstructionArgs & {
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
  'MintNewEditionFromMasterEditionViaTokenInstructionArgs',
);
export type MintNewEditionFromMasterEditionViaTokenInstructionAccounts = {
  newMetadata: web3.PublicKey;
  newEdition: web3.PublicKey;
  masterEdition: web3.PublicKey;
  newMint: web3.PublicKey;
  editionMarkPda: web3.PublicKey;
  newMintAuthority: web3.PublicKey;
  payer: web3.PublicKey;
  tokenAccountOwner: web3.PublicKey;
  tokenAccount: web3.PublicKey;
  newMetadataUpdateAuthority: web3.PublicKey;
  metadata: web3.PublicKey;
  systemAccount: web3.PublicKey;
};

const mintNewEditionFromMasterEditionViaTokenInstructionDiscriminator = [
  252, 218, 191, 168, 126, 69, 125, 118,
];

/**
 * Creates a _MintNewEditionFromMasterEditionViaToken_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createMintNewEditionFromMasterEditionViaTokenInstruction(
  accounts: MintNewEditionFromMasterEditionViaTokenInstructionAccounts,
  args: MintNewEditionFromMasterEditionViaTokenInstructionArgs,
) {
  const {
    newMetadata,
    newEdition,
    masterEdition,
    newMint,
    editionMarkPda,
    newMintAuthority,
    payer,
    tokenAccountOwner,
    tokenAccount,
    newMetadataUpdateAuthority,
    metadata,
    systemAccount,
  } = accounts;

  const [data] = MintNewEditionFromMasterEditionViaTokenStruct.serialize({
    instructionDiscriminator: mintNewEditionFromMasterEditionViaTokenInstructionDiscriminator,
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
      pubkey: tokenAccountOwner,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: tokenAccount,
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
