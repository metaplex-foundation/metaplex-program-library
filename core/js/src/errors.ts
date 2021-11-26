export enum ErrorCode {
  ERROR_INVALID_OWNER,
  ERROR_INVALID_ACCOUNT_DATA,
  ERROR_DEPRECATED_ACCOUNT_DATA,
}

export class MetaplexError extends Error {
  errorCode: ErrorCode;

  constructor(errorCode: ErrorCode, message: string) {
    super(message);
    this.errorCode = errorCode;
  }
}

export const ERROR_INVALID_OWNER: () => MetaplexError = () => {
  return new MetaplexError(ErrorCode.ERROR_INVALID_OWNER, 'Invalid owner');
};

export const ERROR_INVALID_ACCOUNT_DATA: () => MetaplexError = () => {
  return new MetaplexError(ErrorCode.ERROR_INVALID_ACCOUNT_DATA, 'Invalid data');
};

export const ERROR_DEPRECATED_ACCOUNT_DATA: () => MetaplexError = () => {
  return new MetaplexError(ErrorCode.ERROR_DEPRECATED_ACCOUNT_DATA, 'Account data is deprecated');
};
