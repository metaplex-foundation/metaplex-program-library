import { Connection, Transaction } from '@metaplex/mpl-core';
import { MintLayout, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Keypair, PublicKey, SystemProgram, TransactionCtorFields } from '@solana/web3.js';

// from: src/programs/shared/transactions/CreateMint.ts
type CreateMintParams = {
  newAccountPubkey: PublicKey;
  lamports: number;
  decimals?: number;
  owner?: PublicKey;
  freezeAuthority?: PublicKey;
};

export class CreateMint extends Transaction {
  constructor(options: TransactionCtorFields, params: CreateMintParams) {
    const { feePayer } = options;
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
}
// from: src/actions/shared/index.ts
export async function createMintAccount(connection: Connection, payer: PublicKey) {
  const mint = Keypair.generate();

  const mintRent = await connection.getMinimumBalanceForRentExemption(MintLayout.span, 'confirmed');
  const createMintTx = new CreateMint(
    { feePayer: payer },
    {
      newAccountPubkey: mint.publicKey,
      lamports: mintRent,
    },
  );
  return { mint, createMintTx };
}
