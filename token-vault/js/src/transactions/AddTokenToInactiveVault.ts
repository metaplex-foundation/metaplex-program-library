import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
import { AmountArgs } from '../accounts/Vault';
import { Transaction } from '../../../Transaction';
import { VaultProgram } from '../VaultProgram';
import { VaultInstructions } from '../VaultProgram';

type AddTokenToInactiveVaultParams = {
  vault: PublicKey;
  vaultAuthority: PublicKey;
  tokenMint: PublicKey;
  tokenAccount: PublicKey;
  tokenStoreAccount: PublicKey;
  transferAuthority: PublicKey;
  safetyDepositBox: PublicKey;
  amount: BN;
};

export class AddTokenToInactiveVault extends Transaction {
  constructor(options: TransactionCtorFields, params: AddTokenToInactiveVaultParams) {
    super(options);
    const { feePayer } = options;
    const {
      vault,
      vaultAuthority,
      tokenAccount,
      tokenStoreAccount,
      transferAuthority,
      safetyDepositBox,
      amount,
    } = params;

    const data = AmountArgs.serialize({
      instruction: VaultInstructions.AddTokenToInactiveVault,
      amount,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: safetyDepositBox,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: tokenAccount,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: tokenStoreAccount,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: vaultAuthority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: feePayer,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: transferAuthority,
            isSigner: true,
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
          {
            pubkey: SystemProgram.programId,
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
