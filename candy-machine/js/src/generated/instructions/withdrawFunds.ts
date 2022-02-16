import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

const withdrawFundsStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[] /* size: 8 */;
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'WithdrawFundsInstructionArgs',
);
/**
 * Accounts required by the _withdrawFunds_ instruction
 */
export type WithdrawFundsInstructionAccounts = {
  candyMachine: web3.PublicKey;
  authority: web3.PublicKey;
};

const withdrawFundsInstructionDiscriminator = [241, 36, 29, 111, 208, 31, 104, 217];

/**
 * Creates a _WithdrawFunds_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 */
export function createWithdrawFundsInstruction(accounts: WithdrawFundsInstructionAccounts) {
  const { candyMachine, authority } = accounts;

  const [data] = withdrawFundsStruct.serialize({
    instructionDiscriminator: withdrawFundsInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: candyMachine,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: authority,
      isWritable: false,
      isSigner: true,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ'),
    keys,
    data,
  });
  return ix;
}
