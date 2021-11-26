import { ParamsWithStore } from '@metaplex/types';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey, TransactionCtorFields, TransactionInstruction } from '@solana/web3.js';
import BN from 'bn.js';
import { VaultInstructions } from '../VaultProgram';
import { Transaction } from '../../../Transaction';
import { NumberOfShareArgs } from '../accounts/Vault';
import { VaultProgram } from '../VaultProgram';

type MintFractionalSharesParams = {
  vault: PublicKey;
  fractionMint: PublicKey;
  fractionMintAuthority: PublicKey;
  fractionTreasury: PublicKey;
  vaultAuthority: PublicKey;
  numberOfShares: BN;
};

export class MintFractionalShares extends Transaction {
  constructor(options: TransactionCtorFields, params: ParamsWithStore<MintFractionalSharesParams>) {
    super(options);
    const {
      vault,
      vaultAuthority,
      fractionMint,
      fractionTreasury,
      fractionMintAuthority,
      numberOfShares,
    } = params;

    const data = NumberOfShareArgs.serialize({
      instruction: VaultInstructions.MintFractionalShares,
      numberOfShares,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: fractionTreasury,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: fractionMint,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: vault,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: fractionMintAuthority,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: vaultAuthority,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: TOKEN_PROGRAM_ID,
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
