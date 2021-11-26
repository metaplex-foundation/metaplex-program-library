import { ParamsWithStore } from '@metaplex/types';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
import { VaultInstructions } from '../VaultProgram';
import { Transaction } from '../../../Transaction';
import { AmountArgs } from '../accounts/Vault';
import { VaultProgram } from '../VaultProgram';

type WithdrawTokenFromSafetyDepositBoxParams = {
  vault: PublicKey;
  destination: PublicKey;
  safetyDepositBox: PublicKey;
  fractionMint: PublicKey;
  vaultAuthority: PublicKey;
  transferAuthority: PublicKey;
  amount: BN;
};

export class WithdrawTokenFromSafetyDepositBox extends Transaction {
  constructor(
    options: TransactionCtorFields,
    params: ParamsWithStore<WithdrawTokenFromSafetyDepositBoxParams>,
  ) {
    super(options);
    const {
      vault,
      vaultAuthority,
      store,
      destination,
      fractionMint,
      transferAuthority,
      safetyDepositBox,
      amount,
    } = params;

    const data = AmountArgs.serialize({
      instruction: VaultInstructions.WithdrawTokenFromSafetyDepositBox,
      amount,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: destination,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: safetyDepositBox,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: store,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: fractionMint,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: vaultAuthority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: transferAuthority,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
          },
        ],
        programId: VaultProgram.PUBKEY,
        data,
      }),
    );
  }
}
