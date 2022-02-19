import * as splToken from '@solana/spl-token';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

const DeprecatedMintNewEditionFromMasterEditionViaPrintingTokenStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[];
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'DeprecatedMintNewEditionFromMasterEditionViaPrintingTokenInstructionArgs',
);
export type DeprecatedMintNewEditionFromMasterEditionViaPrintingTokenInstructionAccounts = {
  metadata: web3.PublicKey;
  edition: web3.PublicKey;
  masterEdition: web3.PublicKey;
  mint: web3.PublicKey;
  mintAuthority: web3.PublicKey;
  printingMint: web3.PublicKey;
  masterTokenAccount: web3.PublicKey;
  editionMarker: web3.PublicKey;
  burnAuthority: web3.PublicKey;
  payer: web3.PublicKey;
  masterUpdateAuthority: web3.PublicKey;
  masterMetadata: web3.PublicKey;
  systemAccount: web3.PublicKey;
  reservationList: web3.PublicKey;
};

const deprecatedMintNewEditionFromMasterEditionViaPrintingTokenInstructionDiscriminator = [
  154, 36, 174, 111, 190, 80, 155, 228,
];

/**
 * Creates a _DeprecatedMintNewEditionFromMasterEditionViaPrintingToken_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createDeprecatedMintNewEditionFromMasterEditionViaPrintingTokenInstruction(
  accounts: DeprecatedMintNewEditionFromMasterEditionViaPrintingTokenInstructionAccounts,
) {
  const {
    metadata,
    edition,
    masterEdition,
    mint,
    mintAuthority,
    printingMint,
    masterTokenAccount,
    editionMarker,
    burnAuthority,
    payer,
    masterUpdateAuthority,
    masterMetadata,
    systemAccount,
    reservationList,
  } = accounts;

  const [data] = DeprecatedMintNewEditionFromMasterEditionViaPrintingTokenStruct.serialize({
    instructionDiscriminator:
      deprecatedMintNewEditionFromMasterEditionViaPrintingTokenInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: edition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: masterEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: mint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: mintAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: printingMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: masterTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: editionMarker,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: burnAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: payer,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: masterUpdateAuthority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: masterMetadata,
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
    {
      pubkey: reservationList,
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
