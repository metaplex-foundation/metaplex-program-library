import { strict as assert } from 'assert';
import { Connection, Keypair, PublicKey, Signer, TransactionInstruction } from '@solana/web3.js';
import { pdaForMetadata } from '../common/helpers';
import {
  CreateMetadataAccountV2InstructionArgs,
  CreateMetadataAccountV2InstructionAccounts,
  DataV2,
  createCreateMetadataAccountV2Instruction,
} from '../generated';
import { createMintInstructions } from '../common/instructions';

type HasMint = CreateMetadataAccountSetup & {
  mint: PublicKey;
};
type HasMetadata = CreateMetadataAccountSetup & {
  metadata: PublicKey;
};

type CompletedCreateMetadataAccountSetup = CreateMetadataAccountSetup & HasMint & HasMetadata;

/**
 * Used to setup accounts for the {@link createMetadataAccount} instruction.
 *
 * @category Instructions
 * @category CreateMetadataAccountV2
 */
export class CreateMetadataAccountSetup {
  readonly instructions: TransactionInstruction[] = [];
  readonly signers: Signer[] = [];

  /**
   * The initialized mint account for which to create this metadata, provide it
   * or use {@link createMintAccount} in order to initialized it
   */
  mint?: PublicKey;
  /**
   * The metadata PDA whose seeds need to include the mint address.
   * This is automatically set when {@link createMintAccount} is used to
   * initialize the mint account.
   */
  metadata?: PublicKey;

  private constructor(
    private readonly connection: Connection,
    readonly payer: PublicKey,
    readonly updateAuthority: PublicKey,
    readonly mintAuthority: PublicKey,
  ) {}

  /**
   * Creates a {@link CreateMetadataAccountSetup} instance
   *
   * @param args
   * @param args.payer {@link CreateMetadataAccountV2InstructionAccounts} `payer`
   * @param args.updateAuthority {@link CreateMetadataAccountV2InstructionAccounts} `updateAuthority`, defaults to {@link payer}
   * @param args.mintAuthority {@link CreateMetadataAccountV2InstructionAccounts} `mintAuthority`, defaults to {@link payer}
   */
  static create(
    connection: Connection,
    args: { payer: PublicKey; updateAuthority?: PublicKey; mintAuthority?: PublicKey },
  ) {
    const { payer, updateAuthority, mintAuthority } = args;
    return new CreateMetadataAccountSetup(
      connection,
      payer,
      updateAuthority ?? payer,
      mintAuthority ?? payer,
    );
  }

  /**
   * Use this to create and allocate a mint account for the token metadata.
   * If you already have this you can just assign it to the field directly
   * instead, making sure that you also assign the {@link metadata} PDA.
   */
  async createMintAccount({ decimals = 0, owner = this.payer, freezeAuthority = this.payer } = {}) {
    const mint = Keypair.generate();
    const instructions = await createMintInstructions(this.connection, {
      payer: this.payer,
      mint: mint.publicKey,
      decimals,
      owner,
      freezeAuthority,
    });

    this.instructions.push(...instructions);
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

/**
 * Creates a Metadata Account using the accounts provided via the
 * {@link CreateMetadataAccountSetup} and the data inputs.
 *
 * @param setup which is required to have an initialized mint account
 * @param data describing the metadata we want to initialize
 * @param isMutable determines if the metadata can be changed afterwards
 *
 * @category Instructions
 * @category CreateMetadataAccountV2
 */
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
