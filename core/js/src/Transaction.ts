import { Transaction as SolanaTransaction } from '@solana/web3.js';

export class Transaction extends SolanaTransaction {
  constructor(options?: ConstructorParameters<typeof Transaction>[0]) {
    super(options);
  }

  static fromCombined(transactions: Transaction[], options: ConstructorParameters<typeof Transaction>[0] = {}) {
    const combinedTransaction = new Transaction(options);
    transactions.forEach((transaction) =>
      transaction.instructions.forEach((instruction) => {
        combinedTransaction.add(instruction);
      }),
    );
    return combinedTransaction;
  }
}
