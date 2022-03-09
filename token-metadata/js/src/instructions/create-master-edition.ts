import { Connection, Keypair, PublicKey, Signer, TransactionInstruction } from '@solana/web3.js';
import { pdaForMetadata } from '../common/helpers';
import { createMintInstructions } from '../common/instructions';

export class CreateMasterEditionSetup {
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

    return this; // .asCompleted();
  }
}

// function createMasterEdition() {}
