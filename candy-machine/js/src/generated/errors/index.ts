type ErrorWithCode = Error & { code: number };
type MaybeErrorWithCode = ErrorWithCode | null | undefined;

const createErrorFromCodeLookup: Map<number, () => ErrorWithCode> = new Map();
const createErrorFromNameLookup: Map<string, () => ErrorWithCode> = new Map();

/**
 * IncorrectOwner: 'Account does not have correct owner!'
 */
export class IncorrectOwnerError extends Error {
  readonly code: number = 0x1770;
  readonly name: string = 'IncorrectOwner';
  constructor() {
    super('Account does not have correct owner!');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, IncorrectOwnerError);
    }
  }
}

createErrorFromCodeLookup.set(0x1770, () => new IncorrectOwnerError());
createErrorFromNameLookup.set('IncorrectOwner', () => new IncorrectOwnerError());

/**
 * Uninitialized: 'Account is not initialized!'
 */
export class UninitializedError extends Error {
  readonly code: number = 0x1771;
  readonly name: string = 'Uninitialized';
  constructor() {
    super('Account is not initialized!');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, UninitializedError);
    }
  }
}

createErrorFromCodeLookup.set(0x1771, () => new UninitializedError());
createErrorFromNameLookup.set('Uninitialized', () => new UninitializedError());

/**
 * MintMismatch: 'Mint Mismatch!'
 */
export class MintMismatchError extends Error {
  readonly code: number = 0x1772;
  readonly name: string = 'MintMismatch';
  constructor() {
    super('Mint Mismatch!');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, MintMismatchError);
    }
  }
}

createErrorFromCodeLookup.set(0x1772, () => new MintMismatchError());
createErrorFromNameLookup.set('MintMismatch', () => new MintMismatchError());

/**
 * IndexGreaterThanLength: 'Index greater than length!'
 */
export class IndexGreaterThanLengthError extends Error {
  readonly code: number = 0x1773;
  readonly name: string = 'IndexGreaterThanLength';
  constructor() {
    super('Index greater than length!');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, IndexGreaterThanLengthError);
    }
  }
}

createErrorFromCodeLookup.set(0x1773, () => new IndexGreaterThanLengthError());
createErrorFromNameLookup.set('IndexGreaterThanLength', () => new IndexGreaterThanLengthError());

/**
 * NumericalOverflowError: 'Numerical overflow error!'
 */
export class NumericalOverflowErrorError extends Error {
  readonly code: number = 0x1774;
  readonly name: string = 'NumericalOverflowError';
  constructor() {
    super('Numerical overflow error!');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NumericalOverflowErrorError);
    }
  }
}

createErrorFromCodeLookup.set(0x1774, () => new NumericalOverflowErrorError());
createErrorFromNameLookup.set('NumericalOverflowError', () => new NumericalOverflowErrorError());

/**
 * TooManyCreators: 'Can only provide up to 4 creators to candy machine (because candy machine is one)!'
 */
export class TooManyCreatorsError extends Error {
  readonly code: number = 0x1775;
  readonly name: string = 'TooManyCreators';
  constructor() {
    super('Can only provide up to 4 creators to candy machine (because candy machine is one)!');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, TooManyCreatorsError);
    }
  }
}

createErrorFromCodeLookup.set(0x1775, () => new TooManyCreatorsError());
createErrorFromNameLookup.set('TooManyCreators', () => new TooManyCreatorsError());

/**
 * UuidMustBeExactly6Length: 'Uuid must be exactly of 6 length'
 */
export class UuidMustBeExactly6LengthError extends Error {
  readonly code: number = 0x1776;
  readonly name: string = 'UuidMustBeExactly6Length';
  constructor() {
    super('Uuid must be exactly of 6 length');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, UuidMustBeExactly6LengthError);
    }
  }
}

createErrorFromCodeLookup.set(0x1776, () => new UuidMustBeExactly6LengthError());
createErrorFromNameLookup.set(
  'UuidMustBeExactly6Length',
  () => new UuidMustBeExactly6LengthError(),
);

/**
 * NotEnoughTokens: 'Not enough tokens to pay for this minting'
 */
export class NotEnoughTokensError extends Error {
  readonly code: number = 0x1777;
  readonly name: string = 'NotEnoughTokens';
  constructor() {
    super('Not enough tokens to pay for this minting');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NotEnoughTokensError);
    }
  }
}

createErrorFromCodeLookup.set(0x1777, () => new NotEnoughTokensError());
createErrorFromNameLookup.set('NotEnoughTokens', () => new NotEnoughTokensError());

/**
 * NotEnoughSOL: 'Not enough SOL to pay for this minting'
 */
export class NotEnoughSOLError extends Error {
  readonly code: number = 0x1778;
  readonly name: string = 'NotEnoughSOL';
  constructor() {
    super('Not enough SOL to pay for this minting');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NotEnoughSOLError);
    }
  }
}

createErrorFromCodeLookup.set(0x1778, () => new NotEnoughSOLError());
createErrorFromNameLookup.set('NotEnoughSOL', () => new NotEnoughSOLError());

/**
 * TokenTransferFailed: 'Token transfer failed'
 */
export class TokenTransferFailedError extends Error {
  readonly code: number = 0x1779;
  readonly name: string = 'TokenTransferFailed';
  constructor() {
    super('Token transfer failed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, TokenTransferFailedError);
    }
  }
}

createErrorFromCodeLookup.set(0x1779, () => new TokenTransferFailedError());
createErrorFromNameLookup.set('TokenTransferFailed', () => new TokenTransferFailedError());

/**
 * CandyMachineEmpty: 'Candy machine is empty!'
 */
export class CandyMachineEmptyError extends Error {
  readonly code: number = 0x177a;
  readonly name: string = 'CandyMachineEmpty';
  constructor() {
    super('Candy machine is empty!');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, CandyMachineEmptyError);
    }
  }
}

createErrorFromCodeLookup.set(0x177a, () => new CandyMachineEmptyError());
createErrorFromNameLookup.set('CandyMachineEmpty', () => new CandyMachineEmptyError());

/**
 * CandyMachineNotLive: 'Candy machine is not live!'
 */
export class CandyMachineNotLiveError extends Error {
  readonly code: number = 0x177b;
  readonly name: string = 'CandyMachineNotLive';
  constructor() {
    super('Candy machine is not live!');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, CandyMachineNotLiveError);
    }
  }
}

createErrorFromCodeLookup.set(0x177b, () => new CandyMachineNotLiveError());
createErrorFromNameLookup.set('CandyMachineNotLive', () => new CandyMachineNotLiveError());

/**
 * HiddenSettingsConfigsDoNotHaveConfigLines: 'Configs that are using hidden uris do not have config lines, they have a single hash representing hashed order'
 */
export class HiddenSettingsConfigsDoNotHaveConfigLinesError extends Error {
  readonly code: number = 0x177c;
  readonly name: string = 'HiddenSettingsConfigsDoNotHaveConfigLines';
  constructor() {
    super(
      'Configs that are using hidden uris do not have config lines, they have a single hash representing hashed order',
    );
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, HiddenSettingsConfigsDoNotHaveConfigLinesError);
    }
  }
}

createErrorFromCodeLookup.set(0x177c, () => new HiddenSettingsConfigsDoNotHaveConfigLinesError());
createErrorFromNameLookup.set(
  'HiddenSettingsConfigsDoNotHaveConfigLines',
  () => new HiddenSettingsConfigsDoNotHaveConfigLinesError(),
);

/**
 * CannotChangeNumberOfLines: 'Cannot change number of lines unless is a hidden config'
 */
export class CannotChangeNumberOfLinesError extends Error {
  readonly code: number = 0x177d;
  readonly name: string = 'CannotChangeNumberOfLines';
  constructor() {
    super('Cannot change number of lines unless is a hidden config');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, CannotChangeNumberOfLinesError);
    }
  }
}

createErrorFromCodeLookup.set(0x177d, () => new CannotChangeNumberOfLinesError());
createErrorFromNameLookup.set(
  'CannotChangeNumberOfLines',
  () => new CannotChangeNumberOfLinesError(),
);

/**
 * DerivedKeyInvalid: 'Derived key invalid'
 */
export class DerivedKeyInvalidError extends Error {
  readonly code: number = 0x177e;
  readonly name: string = 'DerivedKeyInvalid';
  constructor() {
    super('Derived key invalid');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DerivedKeyInvalidError);
    }
  }
}

createErrorFromCodeLookup.set(0x177e, () => new DerivedKeyInvalidError());
createErrorFromNameLookup.set('DerivedKeyInvalid', () => new DerivedKeyInvalidError());

/**
 * PublicKeyMismatch: 'Public key mismatch'
 */
export class PublicKeyMismatchError extends Error {
  readonly code: number = 0x177f;
  readonly name: string = 'PublicKeyMismatch';
  constructor() {
    super('Public key mismatch');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PublicKeyMismatchError);
    }
  }
}

createErrorFromCodeLookup.set(0x177f, () => new PublicKeyMismatchError());
createErrorFromNameLookup.set('PublicKeyMismatch', () => new PublicKeyMismatchError());

/**
 * NoWhitelistToken: 'No whitelist token present'
 */
export class NoWhitelistTokenError extends Error {
  readonly code: number = 0x1780;
  readonly name: string = 'NoWhitelistToken';
  constructor() {
    super('No whitelist token present');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NoWhitelistTokenError);
    }
  }
}

createErrorFromCodeLookup.set(0x1780, () => new NoWhitelistTokenError());
createErrorFromNameLookup.set('NoWhitelistToken', () => new NoWhitelistTokenError());

/**
 * TokenBurnFailed: 'Token burn failed'
 */
export class TokenBurnFailedError extends Error {
  readonly code: number = 0x1781;
  readonly name: string = 'TokenBurnFailed';
  constructor() {
    super('Token burn failed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, TokenBurnFailedError);
    }
  }
}

createErrorFromCodeLookup.set(0x1781, () => new TokenBurnFailedError());
createErrorFromNameLookup.set('TokenBurnFailed', () => new TokenBurnFailedError());

/**
 * GatewayAppMissing: 'Missing gateway app when required'
 */
export class GatewayAppMissingError extends Error {
  readonly code: number = 0x1782;
  readonly name: string = 'GatewayAppMissing';
  constructor() {
    super('Missing gateway app when required');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, GatewayAppMissingError);
    }
  }
}

createErrorFromCodeLookup.set(0x1782, () => new GatewayAppMissingError());
createErrorFromNameLookup.set('GatewayAppMissing', () => new GatewayAppMissingError());

/**
 * GatewayTokenMissing: 'Missing gateway token when required'
 */
export class GatewayTokenMissingError extends Error {
  readonly code: number = 0x1783;
  readonly name: string = 'GatewayTokenMissing';
  constructor() {
    super('Missing gateway token when required');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, GatewayTokenMissingError);
    }
  }
}

createErrorFromCodeLookup.set(0x1783, () => new GatewayTokenMissingError());
createErrorFromNameLookup.set('GatewayTokenMissing', () => new GatewayTokenMissingError());

/**
 * GatewayTokenExpireTimeInvalid: 'Invalid gateway token expire time'
 */
export class GatewayTokenExpireTimeInvalidError extends Error {
  readonly code: number = 0x1784;
  readonly name: string = 'GatewayTokenExpireTimeInvalid';
  constructor() {
    super('Invalid gateway token expire time');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, GatewayTokenExpireTimeInvalidError);
    }
  }
}

createErrorFromCodeLookup.set(0x1784, () => new GatewayTokenExpireTimeInvalidError());
createErrorFromNameLookup.set(
  'GatewayTokenExpireTimeInvalid',
  () => new GatewayTokenExpireTimeInvalidError(),
);

/**
 * NetworkExpireFeatureMissing: 'Missing gateway network expire feature when required'
 */
export class NetworkExpireFeatureMissingError extends Error {
  readonly code: number = 0x1785;
  readonly name: string = 'NetworkExpireFeatureMissing';
  constructor() {
    super('Missing gateway network expire feature when required');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NetworkExpireFeatureMissingError);
    }
  }
}

createErrorFromCodeLookup.set(0x1785, () => new NetworkExpireFeatureMissingError());
createErrorFromNameLookup.set(
  'NetworkExpireFeatureMissing',
  () => new NetworkExpireFeatureMissingError(),
);

/**
 * CannotFindUsableConfigLine: 'Unable to find an unused config line near your random number index'
 */
export class CannotFindUsableConfigLineError extends Error {
  readonly code: number = 0x1786;
  readonly name: string = 'CannotFindUsableConfigLine';
  constructor() {
    super('Unable to find an unused config line near your random number index');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, CannotFindUsableConfigLineError);
    }
  }
}

createErrorFromCodeLookup.set(0x1786, () => new CannotFindUsableConfigLineError());
createErrorFromNameLookup.set(
  'CannotFindUsableConfigLine',
  () => new CannotFindUsableConfigLineError(),
);

/**
 * InvalidString: 'Invalid string'
 */
export class InvalidStringError extends Error {
  readonly code: number = 0x1787;
  readonly name: string = 'InvalidString';
  constructor() {
    super('Invalid string');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidStringError);
    }
  }
}

createErrorFromCodeLookup.set(0x1787, () => new InvalidStringError());
createErrorFromNameLookup.set('InvalidString', () => new InvalidStringError());

/**
 * SuspiciousTransaction: 'Suspicious transaction detected'
 */
export class SuspiciousTransactionError extends Error {
  readonly code: number = 0x1788;
  readonly name: string = 'SuspiciousTransaction';
  constructor() {
    super('Suspicious transaction detected');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SuspiciousTransactionError);
    }
  }
}

createErrorFromCodeLookup.set(0x1788, () => new SuspiciousTransactionError());
createErrorFromNameLookup.set('SuspiciousTransaction', () => new SuspiciousTransactionError());

/**
 * CannotSwitchToHiddenSettings: 'Cannot Switch to Hidden Settings after items available is greater than 0'
 */
export class CannotSwitchToHiddenSettingsError extends Error {
  readonly code: number = 0x1789;
  readonly name: string = 'CannotSwitchToHiddenSettings';
  constructor() {
    super('Cannot Switch to Hidden Settings after items available is greater than 0');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, CannotSwitchToHiddenSettingsError);
    }
  }
}

createErrorFromCodeLookup.set(0x1789, () => new CannotSwitchToHiddenSettingsError());
createErrorFromNameLookup.set(
  'CannotSwitchToHiddenSettings',
  () => new CannotSwitchToHiddenSettingsError(),
);

/**
 * IncorrectSlotHashesPubkey: 'Incorrect SlotHashes PubKey'
 */
export class IncorrectSlotHashesPubkeyError extends Error {
  readonly code: number = 0x178a;
  readonly name: string = 'IncorrectSlotHashesPubkey';
  constructor() {
    super('Incorrect SlotHashes PubKey');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, IncorrectSlotHashesPubkeyError);
    }
  }
}

createErrorFromCodeLookup.set(0x178a, () => new IncorrectSlotHashesPubkeyError());
createErrorFromNameLookup.set(
  'IncorrectSlotHashesPubkey',
  () => new IncorrectSlotHashesPubkeyError(),
);

/**
 * Attempts to resolve a custom program error from the provided error code.
 */
export function errorFromCode(code: number): MaybeErrorWithCode {
  const createError = createErrorFromCodeLookup.get(code);
  return createError != null ? createError() : null;
}

/**
 * Attempts to resolve a custom program error from the provided error name, i.e. 'Unauthorized'.
 */
export function errorFromName(name: string): MaybeErrorWithCode {
  const createError = createErrorFromNameLookup.get(name);
  return createError != null ? createError() : null;
}
