import { Borsh } from '@metaplex/utils';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
import { Transaction } from '../../../Transaction';
import { MetadataProgram } from '../MetadataProgram';

export class MintNewEditionFromMasterEditionViaTokenArgs extends Borsh.Data<{ edition: BN }> {
  static readonly SCHEMA = this.struct([
    ['instruction', 'u8'],
    ['edition', 'u64'],
  ]);

  instruction = 11;
  edition: BN;
}

type MintNewEditionFromMasterEditionViaTokenParams = {
  edition: PublicKey;
  metadata: PublicKey;
  updateAuthority: PublicKey;
  mint: PublicKey;
  mintAuthority: PublicKey;
  masterEdition: PublicKey;
  masterMetadata: PublicKey;
  editionMarker: PublicKey;
  tokenOwner: PublicKey;
  tokenAccount: PublicKey;
  editionValue: BN;
};

export class MintNewEditionFromMasterEditionViaToken extends Transaction {
  constructor(
    options: TransactionCtorFields,
    params: MintNewEditionFromMasterEditionViaTokenParams,
  ) {
    super(options);
    const { feePayer } = options;
    const {
      edition,
      metadata,
      updateAuthority,
      masterEdition,
      masterMetadata,
      mint,
      editionMarker,
      mintAuthority,
      tokenOwner,
      tokenAccount,
      editionValue,
    } = params;

    const data = MintNewEditionFromMasterEditionViaTokenArgs.serialize({
      edition: editionValue,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: metadata,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: edition,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: masterEdition,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: mint,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: editionMarker,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: mintAuthority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: feePayer,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: tokenOwner,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: tokenAccount,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: updateAuthority,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: masterMetadata,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
          },
        ],
        programId: MetadataProgram.PUBKEY,
        data,
      }),
    );
  }
}
