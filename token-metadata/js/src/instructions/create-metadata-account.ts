import { MintLayout, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { strict as assert } from 'assert';
import {
  Connection,
  Keypair,
  PublicKey,
  Signer,
  SystemProgram,
  TransactionInstruction,
} from '@solana/web3.js';
import { pdaForMetadata } from '../common/helpers';
import {
  CreateMetadataAccountV2InstructionArgs,
  CreateMetadataAccountV2InstructionAccounts,
  DataV2,
  createCreateMetadataAccountV2Instruction,
} from '../generated';

type HasMint = CreateMetadataAccountSetup & {
  mint: PublicKey;
};
type HasMetadata = CreateMetadataAccountSetup & {
  metadata: PublicKey;
};

type CompletedCreateMetadataAccountSetup = CreateMetadataAccountSetup & HasMint & HasMetadata;

export class CreateMetadataAccountSetup {
  readonly instructions: TransactionInstruction[] = [];
  readonly signers: Signer[] = [];

  mint?: PublicKey;
  metadata?: PublicKey;

  private constructor(
    private readonly connection: Connection,
    readonly payer: PublicKey,
    readonly updateAuthority: PublicKey,
    readonly mintAuthority: PublicKey,
  ) {}

  static create(
    connection: Connection,
    {
      payer,
      updateAuthority,
      mintAuthority,
    }: { payer: PublicKey; updateAuthority?: PublicKey; mintAuthority?: PublicKey },
  ) {
    return new CreateMetadataAccountSetup(
      connection,
      payer,
      updateAuthority ?? payer,
      mintAuthority ?? payer,
    );
  }

  async createMintAccount({ decimals = 0, owner = this.payer, freezeAuthority = this.payer } = {}) {
    const mint = Keypair.generate();

    const mintRent = await this.connection.getMinimumBalanceForRentExemption(
      MintLayout.span,
      'confirmed',
    );

    this.instructions.push(
      SystemProgram.createAccount({
        fromPubkey: this.payer,
        newAccountPubkey: mint.publicKey,
        lamports: mintRent,
        space: MintLayout.span,
        programId: TOKEN_PROGRAM_ID,
      }),
    );
    this.instructions.push(
      Token.createInitMintInstruction(
        TOKEN_PROGRAM_ID,
        mint.publicKey,
        decimals,
        owner,
        freezeAuthority,
      ),
    );
    this.signers.push(mint);
    this.mint = mint.publicKey;
    this.metadata = await pdaForMetadata(this.mint);

    return this.asCompleted();
  }

  hasMint(this: CreateMetadataAccountSetup): this is HasMint {
    return this.mint != null;
  }

  hasMetadata(this: CreateMetadataAccountSetup): this is HasMetadata {
    return this.metadata != null;
  }

  assertCompleted(): asserts this is CompletedCreateMetadataAccountSetup {
    assert(this.hasMint(), 'need to provide or create Mint Account first');
    assert(this.hasMetadata(), 'need to provide or create Metadata Account first');
  }

  asCompleted(): CompletedCreateMetadataAccountSetup {
    this.assertCompleted();
    return this;
  }
}

export async function createMetadataAccount(
  setup: CompletedCreateMetadataAccountSetup,
  data: DataV2,
  isMutable: boolean,
) {
  const accounts: CreateMetadataAccountV2InstructionAccounts = {
    mint: setup.mint,
    payer: setup.payer,
    mintAuthority: setup.mintAuthority,
    updateAuthority: setup.updateAuthority,
    metadata: setup.metadata,
  };
  const args: CreateMetadataAccountV2InstructionArgs = {
    createMetadataAccountArgsV2: { data, isMutable },
  };
  const ix = createCreateMetadataAccountV2Instruction(accounts, args);
  return ix;
}
