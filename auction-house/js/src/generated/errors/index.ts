const createErrorFromCodeLookup: Map<number, () => Error> = new Map();
const createErrorFromNameLookup: Map<string, () => Error> = new Map();

export class PublicKeyMismatchError extends Error {
  readonly code: number = 6000;
  readonly name: string = 'PublicKeyMismatch';
  constructor() {
    super('PublicKeyMismatch');
    Error.captureStackTrace(this, PublicKeyMismatchError);
  }
}

createErrorFromCodeLookup.set(6000, () => new PublicKeyMismatchError());
createErrorFromNameLookup.set('PublicKeyMismatch', () => new PublicKeyMismatchError());

export class InvalidMintAuthorityError extends Error {
  readonly code: number = 6001;
  readonly name: string = 'InvalidMintAuthority';
  constructor() {
    super('InvalidMintAuthority');
    Error.captureStackTrace(this, InvalidMintAuthorityError);
  }
}

createErrorFromCodeLookup.set(6001, () => new InvalidMintAuthorityError());
createErrorFromNameLookup.set('InvalidMintAuthority', () => new InvalidMintAuthorityError());

export class UninitializedAccountError extends Error {
  readonly code: number = 6002;
  readonly name: string = 'UninitializedAccount';
  constructor() {
    super('UninitializedAccount');
    Error.captureStackTrace(this, UninitializedAccountError);
  }
}

createErrorFromCodeLookup.set(6002, () => new UninitializedAccountError());
createErrorFromNameLookup.set('UninitializedAccount', () => new UninitializedAccountError());

export class IncorrectOwnerError extends Error {
  readonly code: number = 6003;
  readonly name: string = 'IncorrectOwner';
  constructor() {
    super('IncorrectOwner');
    Error.captureStackTrace(this, IncorrectOwnerError);
  }
}

createErrorFromCodeLookup.set(6003, () => new IncorrectOwnerError());
createErrorFromNameLookup.set('IncorrectOwner', () => new IncorrectOwnerError());

export class PublicKeysShouldBeUniqueError extends Error {
  readonly code: number = 6004;
  readonly name: string = 'PublicKeysShouldBeUnique';
  constructor() {
    super('PublicKeysShouldBeUnique');
    Error.captureStackTrace(this, PublicKeysShouldBeUniqueError);
  }
}

createErrorFromCodeLookup.set(6004, () => new PublicKeysShouldBeUniqueError());
createErrorFromNameLookup.set(
  'PublicKeysShouldBeUnique',
  () => new PublicKeysShouldBeUniqueError(),
);

export class StatementFalseError extends Error {
  readonly code: number = 6005;
  readonly name: string = 'StatementFalse';
  constructor() {
    super('StatementFalse');
    Error.captureStackTrace(this, StatementFalseError);
  }
}

createErrorFromCodeLookup.set(6005, () => new StatementFalseError());
createErrorFromNameLookup.set('StatementFalse', () => new StatementFalseError());

export class NotRentExemptError extends Error {
  readonly code: number = 6006;
  readonly name: string = 'NotRentExempt';
  constructor() {
    super('NotRentExempt');
    Error.captureStackTrace(this, NotRentExemptError);
  }
}

createErrorFromCodeLookup.set(6006, () => new NotRentExemptError());
createErrorFromNameLookup.set('NotRentExempt', () => new NotRentExemptError());

export class NumericalOverflowError extends Error {
  readonly code: number = 6007;
  readonly name: string = 'NumericalOverflow';
  constructor() {
    super('NumericalOverflow');
    Error.captureStackTrace(this, NumericalOverflowError);
  }
}

createErrorFromCodeLookup.set(6007, () => new NumericalOverflowError());
createErrorFromNameLookup.set('NumericalOverflow', () => new NumericalOverflowError());

export class ExpectedSolAccountError extends Error {
  readonly code: number = 6008;
  readonly name: string = 'ExpectedSolAccount';
  constructor() {
    super('Expected a sol account but got an spl token account instead');
    Error.captureStackTrace(this, ExpectedSolAccountError);
  }
}

createErrorFromCodeLookup.set(6008, () => new ExpectedSolAccountError());
createErrorFromNameLookup.set('ExpectedSolAccount', () => new ExpectedSolAccountError());

export class CannotExchangeSOLForSolError extends Error {
  readonly code: number = 6009;
  readonly name: string = 'CannotExchangeSOLForSol';
  constructor() {
    super('Cannot exchange sol for sol');
    Error.captureStackTrace(this, CannotExchangeSOLForSolError);
  }
}

createErrorFromCodeLookup.set(6009, () => new CannotExchangeSOLForSolError());
createErrorFromNameLookup.set('CannotExchangeSOLForSol', () => new CannotExchangeSOLForSolError());

export class SOLWalletMustSignError extends Error {
  readonly code: number = 6010;
  readonly name: string = 'SOLWalletMustSign';
  constructor() {
    super('If paying with sol, sol wallet must be signer');
    Error.captureStackTrace(this, SOLWalletMustSignError);
  }
}

createErrorFromCodeLookup.set(6010, () => new SOLWalletMustSignError());
createErrorFromNameLookup.set('SOLWalletMustSign', () => new SOLWalletMustSignError());

export class CannotTakeThisActionWithoutAuctionHouseSignOffError extends Error {
  readonly code: number = 6011;
  readonly name: string = 'CannotTakeThisActionWithoutAuctionHouseSignOff';
  constructor() {
    super('Cannot take this action without auction house signing too');
    Error.captureStackTrace(this, CannotTakeThisActionWithoutAuctionHouseSignOffError);
  }
}

createErrorFromCodeLookup.set(
  6011,
  () => new CannotTakeThisActionWithoutAuctionHouseSignOffError(),
);
createErrorFromNameLookup.set(
  'CannotTakeThisActionWithoutAuctionHouseSignOff',
  () => new CannotTakeThisActionWithoutAuctionHouseSignOffError(),
);

export class NoPayerPresentError extends Error {
  readonly code: number = 6012;
  readonly name: string = 'NoPayerPresent';
  constructor() {
    super('No payer present on this txn');
    Error.captureStackTrace(this, NoPayerPresentError);
  }
}

createErrorFromCodeLookup.set(6012, () => new NoPayerPresentError());
createErrorFromNameLookup.set('NoPayerPresent', () => new NoPayerPresentError());

export class DerivedKeyInvalidError extends Error {
  readonly code: number = 6013;
  readonly name: string = 'DerivedKeyInvalid';
  constructor() {
    super('Derived key invalid');
    Error.captureStackTrace(this, DerivedKeyInvalidError);
  }
}

createErrorFromCodeLookup.set(6013, () => new DerivedKeyInvalidError());
createErrorFromNameLookup.set('DerivedKeyInvalid', () => new DerivedKeyInvalidError());

export class MetadataDoesntExistError extends Error {
  readonly code: number = 6014;
  readonly name: string = 'MetadataDoesntExist';
  constructor() {
    super("Metadata doesn't exist");
    Error.captureStackTrace(this, MetadataDoesntExistError);
  }
}

createErrorFromCodeLookup.set(6014, () => new MetadataDoesntExistError());
createErrorFromNameLookup.set('MetadataDoesntExist', () => new MetadataDoesntExistError());

export class InvalidTokenAmountError extends Error {
  readonly code: number = 6015;
  readonly name: string = 'InvalidTokenAmount';
  constructor() {
    super('Invalid token amount');
    Error.captureStackTrace(this, InvalidTokenAmountError);
  }
}

createErrorFromCodeLookup.set(6015, () => new InvalidTokenAmountError());
createErrorFromNameLookup.set('InvalidTokenAmount', () => new InvalidTokenAmountError());

export class BothPartiesNeedToAgreeToSaleError extends Error {
  readonly code: number = 6016;
  readonly name: string = 'BothPartiesNeedToAgreeToSale';
  constructor() {
    super('Both parties need to agree to this sale');
    Error.captureStackTrace(this, BothPartiesNeedToAgreeToSaleError);
  }
}

createErrorFromCodeLookup.set(6016, () => new BothPartiesNeedToAgreeToSaleError());
createErrorFromNameLookup.set(
  'BothPartiesNeedToAgreeToSale',
  () => new BothPartiesNeedToAgreeToSaleError(),
);

export class CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoffError extends Error {
  readonly code: number = 6017;
  readonly name: string = 'CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff';
  constructor() {
    super('Cannot match free sales unless the auction house or seller signs off');
    Error.captureStackTrace(this, CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoffError);
  }
}

createErrorFromCodeLookup.set(
  6017,
  () => new CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoffError(),
);
createErrorFromNameLookup.set(
  'CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff',
  () => new CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoffError(),
);

export class SaleRequiresSignerError extends Error {
  readonly code: number = 6018;
  readonly name: string = 'SaleRequiresSigner';
  constructor() {
    super('This sale requires a signer');
    Error.captureStackTrace(this, SaleRequiresSignerError);
  }
}

createErrorFromCodeLookup.set(6018, () => new SaleRequiresSignerError());
createErrorFromNameLookup.set('SaleRequiresSigner', () => new SaleRequiresSignerError());

export class OldSellerNotInitializedError extends Error {
  readonly code: number = 6019;
  readonly name: string = 'OldSellerNotInitialized';
  constructor() {
    super('Old seller not initialized');
    Error.captureStackTrace(this, OldSellerNotInitializedError);
  }
}

createErrorFromCodeLookup.set(6019, () => new OldSellerNotInitializedError());
createErrorFromNameLookup.set('OldSellerNotInitialized', () => new OldSellerNotInitializedError());

export class SellerATACannotHaveDelegateError extends Error {
  readonly code: number = 6020;
  readonly name: string = 'SellerATACannotHaveDelegate';
  constructor() {
    super('Seller ata cannot have a delegate set');
    Error.captureStackTrace(this, SellerATACannotHaveDelegateError);
  }
}

createErrorFromCodeLookup.set(6020, () => new SellerATACannotHaveDelegateError());
createErrorFromNameLookup.set(
  'SellerATACannotHaveDelegate',
  () => new SellerATACannotHaveDelegateError(),
);

export class BuyerATACannotHaveDelegateError extends Error {
  readonly code: number = 6021;
  readonly name: string = 'BuyerATACannotHaveDelegate';
  constructor() {
    super('Buyer ata cannot have a delegate set');
    Error.captureStackTrace(this, BuyerATACannotHaveDelegateError);
  }
}

createErrorFromCodeLookup.set(6021, () => new BuyerATACannotHaveDelegateError());
createErrorFromNameLookup.set(
  'BuyerATACannotHaveDelegate',
  () => new BuyerATACannotHaveDelegateError(),
);

export class NoValidSignerPresentError extends Error {
  readonly code: number = 6022;
  readonly name: string = 'NoValidSignerPresent';
  constructor() {
    super('No valid signer present');
    Error.captureStackTrace(this, NoValidSignerPresentError);
  }
}

createErrorFromCodeLookup.set(6022, () => new NoValidSignerPresentError());
createErrorFromNameLookup.set('NoValidSignerPresent', () => new NoValidSignerPresentError());

export class InvalidBasisPointsError extends Error {
  readonly code: number = 6023;
  readonly name: string = 'InvalidBasisPoints';
  constructor() {
    super('BP must be less than or equal to 10000');
    Error.captureStackTrace(this, InvalidBasisPointsError);
  }
}

createErrorFromCodeLookup.set(6023, () => new InvalidBasisPointsError());
createErrorFromNameLookup.set('InvalidBasisPoints', () => new InvalidBasisPointsError());

export function errorFromCode(code: number): Error | null {
  const createError = createErrorFromCodeLookup.get(code);
  return createError == null ? createError() : null;
}

export function errorFromName(name: string): Error | null {
  const createError = createErrorFromNameLookup.get(name);
  return createError == null ? createError() : null;
}
