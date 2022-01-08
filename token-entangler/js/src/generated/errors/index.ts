type ErrorWithCode = Error & { code: number };
type MaybeErrorWithCode = ErrorWithCode | null | undefined;

const createErrorFromCodeLookup: Map<number, () => ErrorWithCode> = new Map();
const createErrorFromNameLookup: Map<string, () => ErrorWithCode> = new Map();

/**
 * PublicKeyMismatch: 'PublicKeyMismatch'
 */
export class PublicKeyMismatchError extends Error {
  readonly code: number = 0x1770;
  readonly name: string = 'PublicKeyMismatch';
  constructor() {
    super('PublicKeyMismatch');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PublicKeyMismatchError);
    }
  }
}

createErrorFromCodeLookup.set(0x1770, () => new PublicKeyMismatchError());
createErrorFromNameLookup.set('PublicKeyMismatch', () => new PublicKeyMismatchError());

/**
 * InvalidMintAuthority: 'InvalidMintAuthority'
 */
export class InvalidMintAuthorityError extends Error {
  readonly code: number = 0x1771;
  readonly name: string = 'InvalidMintAuthority';
  constructor() {
    super('InvalidMintAuthority');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidMintAuthorityError);
    }
  }
}

createErrorFromCodeLookup.set(0x1771, () => new InvalidMintAuthorityError());
createErrorFromNameLookup.set('InvalidMintAuthority', () => new InvalidMintAuthorityError());

/**
 * UninitializedAccount: 'UninitializedAccount'
 */
export class UninitializedAccountError extends Error {
  readonly code: number = 0x1772;
  readonly name: string = 'UninitializedAccount';
  constructor() {
    super('UninitializedAccount');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, UninitializedAccountError);
    }
  }
}

createErrorFromCodeLookup.set(0x1772, () => new UninitializedAccountError());
createErrorFromNameLookup.set('UninitializedAccount', () => new UninitializedAccountError());

/**
 * IncorrectOwner: 'IncorrectOwner'
 */
export class IncorrectOwnerError extends Error {
  readonly code: number = 0x1773;
  readonly name: string = 'IncorrectOwner';
  constructor() {
    super('IncorrectOwner');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, IncorrectOwnerError);
    }
  }
}

createErrorFromCodeLookup.set(0x1773, () => new IncorrectOwnerError());
createErrorFromNameLookup.set('IncorrectOwner', () => new IncorrectOwnerError());

/**
 * PublicKeysShouldBeUnique: 'PublicKeysShouldBeUnique'
 */
export class PublicKeysShouldBeUniqueError extends Error {
  readonly code: number = 0x1774;
  readonly name: string = 'PublicKeysShouldBeUnique';
  constructor() {
    super('PublicKeysShouldBeUnique');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PublicKeysShouldBeUniqueError);
    }
  }
}

createErrorFromCodeLookup.set(0x1774, () => new PublicKeysShouldBeUniqueError());
createErrorFromNameLookup.set(
  'PublicKeysShouldBeUnique',
  () => new PublicKeysShouldBeUniqueError(),
);

/**
 * StatementFalse: 'StatementFalse'
 */
export class StatementFalseError extends Error {
  readonly code: number = 0x1775;
  readonly name: string = 'StatementFalse';
  constructor() {
    super('StatementFalse');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, StatementFalseError);
    }
  }
}

createErrorFromCodeLookup.set(0x1775, () => new StatementFalseError());
createErrorFromNameLookup.set('StatementFalse', () => new StatementFalseError());

/**
 * NotRentExempt: 'NotRentExempt'
 */
export class NotRentExemptError extends Error {
  readonly code: number = 0x1776;
  readonly name: string = 'NotRentExempt';
  constructor() {
    super('NotRentExempt');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NotRentExemptError);
    }
  }
}

createErrorFromCodeLookup.set(0x1776, () => new NotRentExemptError());
createErrorFromNameLookup.set('NotRentExempt', () => new NotRentExemptError());

/**
 * NumericalOverflow: 'NumericalOverflow'
 */
export class NumericalOverflowError extends Error {
  readonly code: number = 0x1777;
  readonly name: string = 'NumericalOverflow';
  constructor() {
    super('NumericalOverflow');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NumericalOverflowError);
    }
  }
}

createErrorFromCodeLookup.set(0x1777, () => new NumericalOverflowError());
createErrorFromNameLookup.set('NumericalOverflow', () => new NumericalOverflowError());

/**
 * DerivedKeyInvalid: 'Derived key invalid'
 */
export class DerivedKeyInvalidError extends Error {
  readonly code: number = 0x1778;
  readonly name: string = 'DerivedKeyInvalid';
  constructor() {
    super('Derived key invalid');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DerivedKeyInvalidError);
    }
  }
}

createErrorFromCodeLookup.set(0x1778, () => new DerivedKeyInvalidError());
createErrorFromNameLookup.set('DerivedKeyInvalid', () => new DerivedKeyInvalidError());

/**
 * MetadataDoesntExist: 'Metadata doesn't exist'
 */
export class MetadataDoesntExistError extends Error {
  readonly code: number = 0x1779;
  readonly name: string = 'MetadataDoesntExist';
  constructor() {
    super("Metadata doesn't exist");
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, MetadataDoesntExistError);
    }
  }
}

createErrorFromCodeLookup.set(0x1779, () => new MetadataDoesntExistError());
createErrorFromNameLookup.set('MetadataDoesntExist', () => new MetadataDoesntExistError());

/**
 * EditionDoesntExist: 'Edition doesn't exist'
 */
export class EditionDoesntExistError extends Error {
  readonly code: number = 0x177a;
  readonly name: string = 'EditionDoesntExist';
  constructor() {
    super("Edition doesn't exist");
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, EditionDoesntExistError);
    }
  }
}

createErrorFromCodeLookup.set(0x177a, () => new EditionDoesntExistError());
createErrorFromNameLookup.set('EditionDoesntExist', () => new EditionDoesntExistError());

/**
 * InvalidTokenAmount: 'Invalid token amount'
 */
export class InvalidTokenAmountError extends Error {
  readonly code: number = 0x177b;
  readonly name: string = 'InvalidTokenAmount';
  constructor() {
    super('Invalid token amount');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidTokenAmountError);
    }
  }
}

createErrorFromCodeLookup.set(0x177b, () => new InvalidTokenAmountError());
createErrorFromNameLookup.set('InvalidTokenAmount', () => new InvalidTokenAmountError());

/**
 * InvalidMint: 'This token is not a valid mint for this entangled pair'
 */
export class InvalidMintError extends Error {
  readonly code: number = 0x177c;
  readonly name: string = 'InvalidMint';
  constructor() {
    super('This token is not a valid mint for this entangled pair');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidMintError);
    }
  }
}

createErrorFromCodeLookup.set(0x177c, () => new InvalidMintError());
createErrorFromNameLookup.set('InvalidMint', () => new InvalidMintError());

/**
 * EntangledPairExists: 'This pair already exists as it's reverse'
 */
export class EntangledPairExistsError extends Error {
  readonly code: number = 0x177d;
  readonly name: string = 'EntangledPairExists';
  constructor() {
    super("This pair already exists as it's reverse");
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, EntangledPairExistsError);
    }
  }
}

createErrorFromCodeLookup.set(0x177d, () => new EntangledPairExistsError());
createErrorFromNameLookup.set('EntangledPairExists', () => new EntangledPairExistsError());

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
