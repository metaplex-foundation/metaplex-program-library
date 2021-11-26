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
import { NumberOfShareArgs } from '../accounts/Vault';
import { VaultProgram } from '../VaultProgram';

type AddSharesToTreasuryParams = {
  vault: PublicKey;
  source: PublicKey;
  fractionTreasury: PublicKey;
  vaultAuthority: PublicKey;
  transferAuthority: PublicKey;
  numberOfShares: BN;
};

export class AddSharesToTreasury extends Transaction {
  constructor(options: TransactionCtorFields, params: ParamsWithStore<AddSharesToTreasuryParams>) {
    super(options);
    const { vault, vaultAuthority, source, transferAuthority, fractionTreasury, numberOfShares } =
      params;

    const data = NumberOfShareArgs.serialize({
      instruction: VaultInstructions.AddSharesToTreasury,
      numberOfShares,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: source,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: fractionTreasury,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: vault,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: transferAuthority,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: vaultAuthority,
            isSigner: false,
            isWritable: true,
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
