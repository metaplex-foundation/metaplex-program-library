import { Borsh, Transaction } from '@metaplex-foundation/mpl-core';
import { MetadataProgram } from '@metaplex-foundation/mpl-token-metadata';
import { ParamsWithStore } from '@metaplex-foundation/mpl-token-vault';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { MetaplexProgram } from '../MetaplexProgram';
import { SafetyDepositConfigData } from '../mpl-metaplex';

export class ValidateSafetyDepositBoxV2Args extends Borsh.Data<{
  safetyDepositConfig: SafetyDepositConfigData;
}> {
  static readonly SCHEMA = new Map([
    ...SafetyDepositConfigData.SCHEMA,
    ...this.struct([
      ['instruction', 'u8'],
      ['safetyDepositConfig', SafetyDepositConfigData],
    ]),
  ]);
  instruction = 18;
  safetyDepositConfig: SafetyDepositConfigData;
}

type ValidateSafetyDepositBoxV2Params = {
  store: PublicKey;
  vault: PublicKey;
  auctionManager: PublicKey;
  auctionManagerAuthority: PublicKey;
  metadataAuthority: PublicKey;
  originalAuthorityLookup: PublicKey;
  tokenTracker: PublicKey;
  tokenAccount: PublicKey;
  tokenMint: PublicKey;
  edition: PublicKey;
  whitelistedCreator: PublicKey;
  safetyDepositBox: PublicKey;
  safetyDepositTokenStore: PublicKey;
  safetyDepositConfig: PublicKey;
  safetyDepositConfigData: SafetyDepositConfigData;
};

export class ValidateSafetyDepositBoxV2 extends Transaction {
  constructor(
    options: TransactionCtorFields,
    params: ParamsWithStore<ValidateSafetyDepositBoxV2Params>,
  ) {
    super(options);
    const { feePayer } = options;
    const {
      store,
      vault,
      auctionManager,
      auctionManagerAuthority,
      metadataAuthority,
      originalAuthorityLookup,
      tokenTracker,
      tokenAccount,
      tokenMint,
      edition,
      whitelistedCreator,
      safetyDepositBox,
      safetyDepositTokenStore,
      safetyDepositConfig,
      safetyDepositConfigData,
    } = params;

    const data = ValidateSafetyDepositBoxV2Args.serialize({
      safetyDepositConfig: safetyDepositConfigData,
    });

    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: safetyDepositConfig,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: tokenTracker,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: auctionManager,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: tokenAccount,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: originalAuthorityLookup,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: whitelistedCreator,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: store,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: safetyDepositBox,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: safetyDepositTokenStore,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: tokenMint,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: edition,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: vault,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: auctionManagerAuthority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: metadataAuthority,
            isSigner: true,
            isWritable: false,
          },

          {
            pubkey: feePayer,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: MetadataProgram.PUBKEY,
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
        programId: MetaplexProgram.PUBKEY,
        data,
      }),
    );
  }
}
