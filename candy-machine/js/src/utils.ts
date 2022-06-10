import { programs } from '@cardinal/token-manager';
import { utils } from '@project-serum/anchor';
import { ASSOCIATED_TOKEN_PROGRAM_ID, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from './generated';

export const CONFIG_LINE_SIZE = 4 + 32 + 4 + 200;
export const MAX_NAME_LENGTH = 32;
export const MAX_SYMBOL_LENGTH = 10;
export const MAX_CREATOR_LIMIT = 5;
export const MAX_CREATOR_LEN = 32 + 1 + 1;
export const MAX_URI_LENGTH = 200;
export const CONFIG_ARRAY_START =
  8 + // key
  32 + // authority
  32 + //wallet
  33 + // token mint
  4 +
  6 + // uuid
  8 + // price
  8 + // items available
  9 + // go live
  10 + // end settings
  4 +
  MAX_SYMBOL_LENGTH + // u32 len + symbol
  2 + // seller fee basis points
  4 +
  MAX_CREATOR_LIMIT * MAX_CREATOR_LEN + // optional + u32 len + actual vec
  8 + //max supply
  1 + // is mutable
  1 + // retain authority
  1 + // option for hidden setting
  4 +
  MAX_NAME_LENGTH + // name length,
  4 +
  MAX_URI_LENGTH + // uri length,
  32 + // hash
  4 + // max number of lines;
  8 + // items redeemed
  1 + // whitelist option
  1 + // whitelist mint mode
  1 + // allow presale
  9 + // discount price
  32 + // mint key for whitelist
  1 +
  32 +
  1; // gatekeeper

const LOCKUP_SETTINGS_SEED = 'lockup_settings';
export const findLockupSettingsId = async (
  candyMachineId: PublicKey,
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [utils.bytes.utf8.encode(LOCKUP_SETTINGS_SEED), candyMachineId.toBuffer()],
    PROGRAM_ID,
  );
};

export const remainingAccountsForLockup = async (
  candyMachineId: PublicKey,
  mintId: PublicKey,
  userTokenAccountId: PublicKey,
) => {
  const [lockupSettingsId] = await findLockupSettingsId(candyMachineId);
  const [tokenManagerId] = await programs.tokenManager.pda.findTokenManagerAddress(mintId);
  const tokenManagerTokenAccountId = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    mintId,
    tokenManagerId,
    true,
  );
  const [mintCounterId] = await programs.tokenManager.pda.findMintCounterId(mintId);
  const [timeInvalidatorId] = await programs.timeInvalidator.pda.findTimeInvalidatorAddress(
    tokenManagerId,
  );
  return [
    {
      pubkey: lockupSettingsId,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: tokenManagerId,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: tokenManagerTokenAccountId,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: mintCounterId,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: userTokenAccountId,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: programs.tokenManager.TOKEN_MANAGER_ADDRESS,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: timeInvalidatorId,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: programs.timeInvalidator.TIME_INVALIDATOR_ADDRESS,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
  ];
};
