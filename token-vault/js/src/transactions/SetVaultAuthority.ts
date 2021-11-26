import { Borsh } from '@metaplex/utils';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import { VaultInstructions } from '../VaultProgram';
import { Transaction } from '../../../Transaction';
import { VaultProgram } from '../VaultProgram';

export class SetVaultAuthorityArgs extends Borsh.Data {
  static readonly SCHEMA = this.struct([['instruction', 'u8']]);

  instruction = VaultInstructions.SetVaultAuthority;
}

type SetVaultAuthorityParams = {
  vault: PublicKey;
  currentAuthority: PublicKey;
  newAuthority: PublicKey;
};

export class SetVaultAuthority extends Transaction {
  constructor(options: TransactionCtorFields, params: SetVaultAuthorityParams) {
    super(options);
    const { vault, currentAuthority, newAuthority } = params;

    const data = SetVaultAuthorityArgs.serialize();

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: currentAuthority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: newAuthority,
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
