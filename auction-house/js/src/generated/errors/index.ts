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
 * ExpectedSolAccount: 'Expected a sol account but got an spl token account instead'
 */
export class ExpectedSolAccountError extends Error {
  readonly code: number = 0x1778;
  readonly name: string = 'ExpectedSolAccount';
  constructor() {
    super('Expected a sol account but got an spl token account instead');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, ExpectedSolAccountError);
    }
  }
}

createErrorFromCodeLookup.set(0x1778, () => new ExpectedSolAccountError());
createErrorFromNameLookup.set('ExpectedSolAccount', () => new ExpectedSolAccountError());

/**
 * CannotExchangeSOLForSol: 'Cannot exchange sol for sol'
 */
export class CannotExchangeSOLForSolError extends Error {
  readonly code: number = 0x1779;
  readonly name: string = 'CannotExchangeSOLForSol';
  constructor() {
    super('Cannot exchange sol for sol');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, CannotExchangeSOLForSolError);
    }
  }
}

createErrorFromCodeLookup.set(0x1779, () => new CannotExchangeSOLForSolError());
createErrorFromNameLookup.set('CannotExchangeSOLForSol', () => new CannotExchangeSOLForSolError());

/**
 * SOLWalletMustSign: 'If paying with sol, sol wallet must be signer'
 */
export class SOLWalletMustSignError extends Error {
  readonly code: number = 0x177a;
  readonly name: string = 'SOLWalletMustSign';
  constructor() {
    super('If paying with sol, sol wallet must be signer');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SOLWalletMustSignError);
    }
  }
}

createErrorFromCodeLookup.set(0x177a, () => new SOLWalletMustSignError());
createErrorFromNameLookup.set('SOLWalletMustSign', () => new SOLWalletMustSignError());

/**
 * CannotTakeThisActionWithoutAuctionHouseSignOff: 'Cannot take this action without auction house signing too'
 */
export class CannotTakeThisActionWithoutAuctionHouseSignOffError extends Error {
  readonly code: number = 0x177b;
  readonly name: string = 'CannotTakeThisActionWithoutAuctionHouseSignOff';
  constructor() {
    super('Cannot take this action without auction house signing too');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, CannotTakeThisActionWithoutAuctionHouseSignOffError);
    }
  }
}

createErrorFromCodeLookup.set(
  0x177b,
  () => new CannotTakeThisActionWithoutAuctionHouseSignOffError(),
);
createErrorFromNameLookup.set(
  'CannotTakeThisActionWithoutAuctionHouseSignOff',
  () => new CannotTakeThisActionWithoutAuctionHouseSignOffError(),
);

/**
 * NoPayerPresent: 'No payer present on this txn'
 */
export class NoPayerPresentError extends Error {
  readonly code: number = 0x177c;
  readonly name: string = 'NoPayerPresent';
  constructor() {
    super('No payer present on this txn');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NoPayerPresentError);
    }
  }
}

createErrorFromCodeLookup.set(0x177c, () => new NoPayerPresentError());
createErrorFromNameLookup.set('NoPayerPresent', () => new NoPayerPresentError());

/**
 * DerivedKeyInvalid: 'Derived key invalid'
 */
export class DerivedKeyInvalidError extends Error {
  readonly code: number = 0x177d;
  readonly name: string = 'DerivedKeyInvalid';
  constructor() {
    super('Derived key invalid');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DerivedKeyInvalidError);
    }
  }
}

createErrorFromCodeLookup.set(0x177d, () => new DerivedKeyInvalidError());
createErrorFromNameLookup.set('DerivedKeyInvalid', () => new DerivedKeyInvalidError());

/**
 * MetadataDoesntExist: 'Metadata doesn't exist'
 */
export class MetadataDoesntExistError extends Error {
  readonly code: number = 0x177e;
  readonly name: string = 'MetadataDoesntExist';
  constructor() {
    super("Metadata doesn't exist");
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, MetadataDoesntExistError);
    }
  }
}

createErrorFromCodeLookup.set(0x177e, () => new MetadataDoesntExistError());
createErrorFromNameLookup.set('MetadataDoesntExist', () => new MetadataDoesntExistError());

/**
 * InvalidTokenAmount: 'Invalid token amount'
 */
export class InvalidTokenAmountError extends Error {
  readonly code: number = 0x177f;
  readonly name: string = 'InvalidTokenAmount';
  constructor() {
    super('Invalid token amount');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidTokenAmountError);
    }
  }
}

createErrorFromCodeLookup.set(0x177f, () => new InvalidTokenAmountError());
createErrorFromNameLookup.set('InvalidTokenAmount', () => new InvalidTokenAmountError());

/**
 * BothPartiesNeedToAgreeToSale: 'Both parties need to agree to this sale'
 */
export class BothPartiesNeedToAgreeToSaleError extends Error {
  readonly code: number = 0x1780;
  readonly name: string = 'BothPartiesNeedToAgreeToSale';
  constructor() {
    super('Both parties need to agree to this sale');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, BothPartiesNeedToAgreeToSaleError);
    }
  }
}

createErrorFromCodeLookup.set(0x1780, () => new BothPartiesNeedToAgreeToSaleError());
createErrorFromNameLookup.set(
  'BothPartiesNeedToAgreeToSale',
  () => new BothPartiesNeedToAgreeToSaleError(),
);

/**
 * CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff: 'Cannot match free sales unless the auction house or seller signs off'
 */
export class CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoffError extends Error {
  readonly code: number = 0x1781;
  readonly name: string = 'CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff';
  constructor() {
    super('Cannot match free sales unless the auction house or seller signs off');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoffError);
    }
  }
}

createErrorFromCodeLookup.set(
  0x1781,
  () => new CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoffError(),
);
createErrorFromNameLookup.set(
  'CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff',
  () => new CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoffError(),
);

/**
 * SaleRequiresSigner: 'This sale requires a signer'
 */
export class SaleRequiresSignerError extends Error {
  readonly code: number = 0x1782;
  readonly name: string = 'SaleRequiresSigner';
  constructor() {
    super('This sale requires a signer');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SaleRequiresSignerError);
    }
  }
}

createErrorFromCodeLookup.set(0x1782, () => new SaleRequiresSignerError());
createErrorFromNameLookup.set('SaleRequiresSigner', () => new SaleRequiresSignerError());

/**
 * OldSellerNotInitialized: 'Old seller not initialized'
 */
export class OldSellerNotInitializedError extends Error {
  readonly code: number = 0x1783;
  readonly name: string = 'OldSellerNotInitialized';
  constructor() {
    super('Old seller not initialized');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, OldSellerNotInitializedError);
    }
  }
}

createErrorFromCodeLookup.set(0x1783, () => new OldSellerNotInitializedError());
createErrorFromNameLookup.set('OldSellerNotInitialized', () => new OldSellerNotInitializedError());

/**
 * SellerATACannotHaveDelegate: 'Seller ata cannot have a delegate set'
 */
export class SellerATACannotHaveDelegateError extends Error {
  readonly code: number = 0x1784;
  readonly name: string = 'SellerATACannotHaveDelegate';
  constructor() {
    super('Seller ata cannot have a delegate set');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SellerATACannotHaveDelegateError);
    }
  }
}

createErrorFromCodeLookup.set(0x1784, () => new SellerATACannotHaveDelegateError());
createErrorFromNameLookup.set(
  'SellerATACannotHaveDelegate',
  () => new SellerATACannotHaveDelegateError(),
);

/**
 * BuyerATACannotHaveDelegate: 'Buyer ata cannot have a delegate set'
 */
export class BuyerATACannotHaveDelegateError extends Error {
  readonly code: number = 0x1785;
  readonly name: string = 'BuyerATACannotHaveDelegate';
  constructor() {
    super('Buyer ata cannot have a delegate set');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, BuyerATACannotHaveDelegateError);
    }
  }
}

createErrorFromCodeLookup.set(0x1785, () => new BuyerATACannotHaveDelegateError());
createErrorFromNameLookup.set(
  'BuyerATACannotHaveDelegate',
  () => new BuyerATACannotHaveDelegateError(),
);

/**
 * NoValidSignerPresent: 'No valid signer present'
 */
export class NoValidSignerPresentError extends Error {
  readonly code: number = 0x1786;
  readonly name: string = 'NoValidSignerPresent';
  constructor() {
    super('No valid signer present');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NoValidSignerPresentError);
    }
  }
}

createErrorFromCodeLookup.set(0x1786, () => new NoValidSignerPresentError());
createErrorFromNameLookup.set('NoValidSignerPresent', () => new NoValidSignerPresentError());

/**
 * InvalidBasisPoints: 'BP must be less than or equal to 10000'
 */
export class InvalidBasisPointsError extends Error {
  readonly code: number = 0x1787;
  readonly name: string = 'InvalidBasisPoints';
  constructor() {
    super('BP must be less than or equal to 10000');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidBasisPointsError);
    }
  }
}

createErrorFromCodeLookup.set(0x1787, () => new InvalidBasisPointsError());
createErrorFromNameLookup.set('InvalidBasisPoints', () => new InvalidBasisPointsError());

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
