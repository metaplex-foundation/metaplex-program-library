import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { PROGRAM_ID } from '../../consts';

export type WithdrawInstructionArgs = {
  treasuryOwnerBump: number;
  payoutTicketBump: number;
};
const withdrawStruct = new beet.BeetArgsStruct<
  WithdrawInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['treasuryOwnerBump', beet.u8],
    ['payoutTicketBump', beet.u8],
  ],
  'WithdrawInstructionArgs',
);
export type WithdrawInstructionAccounts = {
  market: web3.PublicKey;
  sellingResource: web3.PublicKey;
  metadata: web3.PublicKey;
  treasuryHolder: web3.PublicKey;
  treasuryMint: web3.PublicKey;
  owner: web3.PublicKey;
  destination: web3.PublicKey;
  funder: web3.PublicKey;
  payer: web3.PublicKey;
  payoutTicket: web3.PublicKey;
  associatedTokenProgram: web3.PublicKey;
  primaryMetadataCreators?: web3.PublicKey;
};

const withdrawInstructionDiscriminator = [183, 18, 70, 156, 148, 109, 161, 34];

/**
 * Creates a _Withdraw_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createWithdrawInstruction(
  accounts: WithdrawInstructionAccounts,
  args: WithdrawInstructionArgs,
) {
  const {
    market,
    sellingResource,
    metadata,
    treasuryHolder,
    treasuryMint,
    owner,
    destination,
    funder,
    payer,
    payoutTicket,
    associatedTokenProgram,
    primaryMetadataCreators,
  } = accounts;

  const [data] = withdrawStruct.serialize({
    instructionDiscriminator: withdrawInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: market,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: sellingResource,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: treasuryHolder,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: treasuryMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: owner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: destination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: funder,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: payer,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: payoutTicket,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: web3.SYSVAR_RENT_PUBKEY,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: web3.SYSVAR_CLOCK_PUBKEY,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: associatedTokenProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
  ];

  if (primaryMetadataCreators) {
    keys.push({
      pubkey: primaryMetadataCreators,
      isWritable: false,
      isSigner: false,
    });
  }

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey(PROGRAM_ID),
    keys,
    data,
  });
  return ix;
}
