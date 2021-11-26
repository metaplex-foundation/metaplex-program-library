import { ParamsWithStore } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { VaultInstructions } from '../VaultProgram';
import { Transaction } from '../../../Transaction';
import { ExternalPriceAccountData } from '../accounts/ExternalPriceAccount';
import { VaultProgram } from '../VaultProgram';

export class UpdateExternalPriceAccountArgs extends Borsh.Data<{
  externalPriceAccount: ExternalPriceAccountData;
}> {
  static readonly SCHEMA = new Map([
    ...ExternalPriceAccountData.SCHEMA,
    ...this.struct([['instruction', 'u8']]),
  ]);

  instruction = VaultInstructions.UpdateExternalPriceAccount;
  externalPriceAccount: ExternalPriceAccountData;
}

type UpdateExternalPriceAccountParams = {
  externalPriceAccount: PublicKey;
  externalPriceAccountData: ExternalPriceAccountData;
};

export class UpdateExternalPriceAccount extends Transaction {
  constructor(
    options: TransactionCtorFields,
    params: ParamsWithStore<UpdateExternalPriceAccountParams>,
  ) {
    super(options);
    const { externalPriceAccount, externalPriceAccountData } = params;

    const data = UpdateExternalPriceAccountArgs.serialize({
      externalPriceAccount: externalPriceAccountData,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: externalPriceAccount,
            isSigner: false,
            isWritable: true,
          },
        ],
        programId: VaultProgram.PUBKEY,
        data,
      }),
    );
  }
}
