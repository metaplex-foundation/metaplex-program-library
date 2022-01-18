type ErrorWithCode = Error & { code: number };
type MaybeErrorWithCode = ErrorWithCode | null | undefined;

const createErrorFromCodeLookup: Map<number, () => ErrorWithCode> = new Map();
const createErrorFromNameLookup: Map<string, () => ErrorWithCode> = new Map();

/**
 * NoValidSignerPresent: 'No valid signer present'
 */
export class NoValidSignerPresentError extends Error {
  readonly code: number = 0x1770;
  readonly name: string = 'NoValidSignerPresent';
  constructor() {
    super('No valid signer present');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NoValidSignerPresentError);
    }
  }
}

createErrorFromCodeLookup.set(0x1770, () => new NoValidSignerPresentError());
createErrorFromNameLookup.set('NoValidSignerPresent', () => new NoValidSignerPresentError());

/**
 * StringIsTooLong: 'Some string variable is longer than allowed'
 */
export class StringIsTooLongError extends Error {
  readonly code: number = 0x1771;
  readonly name: string = 'StringIsTooLong';
  constructor() {
    super('Some string variable is longer than allowed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, StringIsTooLongError);
    }
  }
}

createErrorFromCodeLookup.set(0x1771, () => new StringIsTooLongError());
createErrorFromNameLookup.set('StringIsTooLong', () => new StringIsTooLongError());

/**
 * NameIsTooLong: 'Name string variable is longer than allowed'
 */
export class NameIsTooLongError extends Error {
  readonly code: number = 0x1772;
  readonly name: string = 'NameIsTooLong';
  constructor() {
    super('Name string variable is longer than allowed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NameIsTooLongError);
    }
  }
}

createErrorFromCodeLookup.set(0x1772, () => new NameIsTooLongError());
createErrorFromNameLookup.set('NameIsTooLong', () => new NameIsTooLongError());

/**
 * DescriptionIsTooLong: 'Description string variable is longer than allowed'
 */
export class DescriptionIsTooLongError extends Error {
  readonly code: number = 0x1773;
  readonly name: string = 'DescriptionIsTooLong';
  constructor() {
    super('Description string variable is longer than allowed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DescriptionIsTooLongError);
    }
  }
}

createErrorFromCodeLookup.set(0x1773, () => new DescriptionIsTooLongError());
createErrorFromNameLookup.set('DescriptionIsTooLong', () => new DescriptionIsTooLongError());

/**
 * SupplyIsGtThanAvailable: 'Provided supply is gt than available'
 */
export class SupplyIsGtThanAvailableError extends Error {
  readonly code: number = 0x1774;
  readonly name: string = 'SupplyIsGtThanAvailable';
  constructor() {
    super('Provided supply is gt than available');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SupplyIsGtThanAvailableError);
    }
  }
}

createErrorFromCodeLookup.set(0x1774, () => new SupplyIsGtThanAvailableError());
createErrorFromNameLookup.set('SupplyIsGtThanAvailable', () => new SupplyIsGtThanAvailableError());

/**
 * SupplyIsNotProvided: 'Supply is not provided'
 */
export class SupplyIsNotProvidedError extends Error {
  readonly code: number = 0x1775;
  readonly name: string = 'SupplyIsNotProvided';
  constructor() {
    super('Supply is not provided');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SupplyIsNotProvidedError);
    }
  }
}

createErrorFromCodeLookup.set(0x1775, () => new SupplyIsNotProvidedError());
createErrorFromNameLookup.set('SupplyIsNotProvided', () => new SupplyIsNotProvidedError());

/**
 * DerivedKeyInvalid: 'Derived key invalid'
 */
export class DerivedKeyInvalidError extends Error {
  readonly code: number = 0x1776;
  readonly name: string = 'DerivedKeyInvalid';
  constructor() {
    super('Derived key invalid');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DerivedKeyInvalidError);
    }
  }
}

createErrorFromCodeLookup.set(0x1776, () => new DerivedKeyInvalidError());
createErrorFromNameLookup.set('DerivedKeyInvalid', () => new DerivedKeyInvalidError());

/**
 * SellingResourceOwnerInvalid: 'Invalid selling resource owner provided'
 */
export class SellingResourceOwnerInvalidError extends Error {
  readonly code: number = 0x1777;
  readonly name: string = 'SellingResourceOwnerInvalid';
  constructor() {
    super('Invalid selling resource owner provided');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SellingResourceOwnerInvalidError);
    }
  }
}

createErrorFromCodeLookup.set(0x1777, () => new SellingResourceOwnerInvalidError());
createErrorFromNameLookup.set(
  'SellingResourceOwnerInvalid',
  () => new SellingResourceOwnerInvalidError(),
);

/**
 * PublicKeyMismatch: 'PublicKeyMismatch'
 */
export class PublicKeyMismatchError extends Error {
  readonly code: number = 0x1778;
  readonly name: string = 'PublicKeyMismatch';
  constructor() {
    super('PublicKeyMismatch');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PublicKeyMismatchError);
    }
  }
}

createErrorFromCodeLookup.set(0x1778, () => new PublicKeyMismatchError());
createErrorFromNameLookup.set('PublicKeyMismatch', () => new PublicKeyMismatchError());

/**
 * PiecesInOneWalletIsTooMuch: 'Pieces in one wallet cannot be greater than Max Supply value'
 */
export class PiecesInOneWalletIsTooMuchError extends Error {
  readonly code: number = 0x1779;
  readonly name: string = 'PiecesInOneWalletIsTooMuch';
  constructor() {
    super('Pieces in one wallet cannot be greater than Max Supply value');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PiecesInOneWalletIsTooMuchError);
    }
  }
}

createErrorFromCodeLookup.set(0x1779, () => new PiecesInOneWalletIsTooMuchError());
createErrorFromNameLookup.set(
  'PiecesInOneWalletIsTooMuch',
  () => new PiecesInOneWalletIsTooMuchError(),
);

/**
 * StartDateIsInPast: 'StartDate cannot be in the past'
 */
export class StartDateIsInPastError extends Error {
  readonly code: number = 0x177a;
  readonly name: string = 'StartDateIsInPast';
  constructor() {
    super('StartDate cannot be in the past');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, StartDateIsInPastError);
    }
  }
}

createErrorFromCodeLookup.set(0x177a, () => new StartDateIsInPastError());
createErrorFromNameLookup.set('StartDateIsInPast', () => new StartDateIsInPastError());

/**
 * EndDateIsEarlierThanBeginDate: 'EndDate should not be earlier than StartDate'
 */
export class EndDateIsEarlierThanBeginDateError extends Error {
  readonly code: number = 0x177b;
  readonly name: string = 'EndDateIsEarlierThanBeginDate';
  constructor() {
    super('EndDate should not be earlier than StartDate');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, EndDateIsEarlierThanBeginDateError);
    }
  }
}

createErrorFromCodeLookup.set(0x177b, () => new EndDateIsEarlierThanBeginDateError());
createErrorFromNameLookup.set(
  'EndDateIsEarlierThanBeginDate',
  () => new EndDateIsEarlierThanBeginDateError(),
);

/**
 * IncorrectOwner: 'Incorrect account owner'
 */
export class IncorrectOwnerError extends Error {
  readonly code: number = 0x177c;
  readonly name: string = 'IncorrectOwner';
  constructor() {
    super('Incorrect account owner');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, IncorrectOwnerError);
    }
  }
}

createErrorFromCodeLookup.set(0x177c, () => new IncorrectOwnerError());
createErrorFromNameLookup.set('IncorrectOwner', () => new IncorrectOwnerError());

/**
 * MarketIsNotStarted: 'Market is not started'
 */
export class MarketIsNotStartedError extends Error {
  readonly code: number = 0x177d;
  readonly name: string = 'MarketIsNotStarted';
  constructor() {
    super('Market is not started');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, MarketIsNotStartedError);
    }
  }
}

createErrorFromCodeLookup.set(0x177d, () => new MarketIsNotStartedError());
createErrorFromNameLookup.set('MarketIsNotStarted', () => new MarketIsNotStartedError());

/**
 * MarketIsEnded: 'Market is ended'
 */
export class MarketIsEndedError extends Error {
  readonly code: number = 0x177e;
  readonly name: string = 'MarketIsEnded';
  constructor() {
    super('Market is ended');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, MarketIsEndedError);
    }
  }
}

createErrorFromCodeLookup.set(0x177e, () => new MarketIsEndedError());
createErrorFromNameLookup.set('MarketIsEnded', () => new MarketIsEndedError());

/**
 * UserReachBuyLimit: 'User reach buy limit'
 */
export class UserReachBuyLimitError extends Error {
  readonly code: number = 0x177f;
  readonly name: string = 'UserReachBuyLimit';
  constructor() {
    super('User reach buy limit');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, UserReachBuyLimitError);
    }
  }
}

createErrorFromCodeLookup.set(0x177f, () => new UserReachBuyLimitError());
createErrorFromNameLookup.set('UserReachBuyLimit', () => new UserReachBuyLimitError());

/**
 * MathOverflow: 'Math overflow'
 */
export class MathOverflowError extends Error {
  readonly code: number = 0x1780;
  readonly name: string = 'MathOverflow';
  constructor() {
    super('Math overflow');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, MathOverflowError);
    }
  }
}

createErrorFromCodeLookup.set(0x1780, () => new MathOverflowError());
createErrorFromNameLookup.set('MathOverflow', () => new MathOverflowError());

/**
 * SupplyIsGtThanMaxSupply: 'Supply is gt than max supply'
 */
export class SupplyIsGtThanMaxSupplyError extends Error {
  readonly code: number = 0x1781;
  readonly name: string = 'SupplyIsGtThanMaxSupply';
  constructor() {
    super('Supply is gt than max supply');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SupplyIsGtThanMaxSupplyError);
    }
  }
}

createErrorFromCodeLookup.set(0x1781, () => new SupplyIsGtThanMaxSupplyError());
createErrorFromNameLookup.set('SupplyIsGtThanMaxSupply', () => new SupplyIsGtThanMaxSupplyError());

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
