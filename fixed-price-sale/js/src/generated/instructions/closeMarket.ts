/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * @category Instructions
 * @category CloseMarket
 * @category generated
 */
export const closeMarketStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[] /* size: 8 */;
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'CloseMarketInstructionArgs',
);
/**
 * Accounts required by the _closeMarket_ instruction
 *
 * @property [_writable_] market
 * @property [**signer**] owner
 * @property [] clock
 * @category Instructions
 * @category CloseMarket
 * @category generated
 */
export type CloseMarketInstructionAccounts = {
  market: web3.PublicKey;
  owner: web3.PublicKey;
  clock: web3.PublicKey;
  anchorRemainingAccounts?: web3.AccountMeta[];
};

export const closeMarketInstructionDiscriminator = [88, 154, 248, 186, 48, 14, 123, 244];

/**
 * Creates a _CloseMarket_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category CloseMarket
 * @category generated
 */
export function createCloseMarketInstruction(
  accounts: CloseMarketInstructionAccounts,
  programId = new web3.PublicKey('SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo'),
) {
  const [data] = closeMarketStruct.serialize({
    instructionDiscriminator: closeMarketInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.market,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.owner,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.clock,
      isWritable: false,
      isSigner: false,
    },
  ];

  if (accounts.anchorRemainingAccounts != null) {
    for (const acc of accounts.anchorRemainingAccounts) {
      keys.push(acc);
    }
  }

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}
