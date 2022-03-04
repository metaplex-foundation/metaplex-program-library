import { strict as assert } from 'assert';
import { MintLayout, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionCtorFields,
} from '@solana/web3.js';

type CreateMintParams = {
  newAccountPubkey: PublicKey;
  lamports: number;
  decimals?: number;
  owner?: PublicKey;
  freezeAuthority?: PublicKey;
};

/**
 * Transaction that is used to create a mint.
 */
export class CreateMint extends Transaction {
  private constructor(options: TransactionCtorFields, params: CreateMintParams) {
    const { feePayer } = options;
    assert(feePayer != null, 'need to provide non-null feePayer');

    const { newAccountPubkey, lamports, decimals, owner, freezeAuthority } = params;

    super(options);

    this.add(
      SystemProgram.createAccount({
        fromPubkey: feePayer,
        newAccountPubkey,
        lamports,
        space: MintLayout.span,
        programId: TOKEN_PROGRAM_ID,
      }),
    );

    this.add(
      Token.createInitMintInstruction(
        TOKEN_PROGRAM_ID,
        newAccountPubkey,
        decimals ?? 0,
        owner ?? feePayer,
        freezeAuthority ?? feePayer,
      ),
    );
  }

  static async createMintAccount(connection: Connection, payer: PublicKey) {
    const mint = Keypair.generate();

    const mintRent = await connection.getMinimumBalanceForRentExemption(
      MintLayout.span,
      'confirmed',
    );
    const createMintTx = new CreateMint(
      { feePayer: payer },
      {
        newAccountPubkey: mint.publicKey,
        lamports: mintRent,
      },
    );
    return { mint, createMintTx };
  }
}
