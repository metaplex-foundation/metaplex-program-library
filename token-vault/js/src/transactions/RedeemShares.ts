import { Borsh } from '@metaplex/utils';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { VaultInstructions } from '../VaultProgram';
import { Transaction } from '../../../Transaction';
import { VaultProgram } from '../VaultProgram';

export class RedeemSharesArgs extends Borsh.Data {
  static readonly SCHEMA = this.struct([['instruction', 'u8']]);

  instruction = VaultInstructions.RedeemShares;
}

type RedeemSharsParams = {
  burnAuthority: PublicKey;
  fractionMint: PublicKey;
  outstandingSharesAccount: PublicKey;
  proceedsAccount: PublicKey;
  redeemTreasury: PublicKey;
  transferAuthority: PublicKey;
  vault: PublicKey;
};

export class RedeemShares extends Transaction {
  constructor(options: TransactionCtorFields, params: RedeemSharsParams) {
    super(options);
    const {
      vault,
      burnAuthority,
      fractionMint,
      outstandingSharesAccount,
      proceedsAccount,
      redeemTreasury,
      transferAuthority,
    } = params;

    const data = RedeemSharesArgs.serialize();

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: outstandingSharesAccount,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: proceedsAccount,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: fractionMint,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: redeemTreasury,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: transferAuthority,
            isSigner: false,
            isWritable: false,
          },

          {
            pubkey: burnAuthority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: vault,
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
