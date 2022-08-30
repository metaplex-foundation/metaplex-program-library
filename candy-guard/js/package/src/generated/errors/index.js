"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.errorFromName = exports.errorFromCode = exports.AfterEndSettingsMintAmountError = exports.AfterEndSettingsDateError = exports.GatewayTokenInvalidError = exports.NoWhitelistTokenError = exports.TokenBurnFailedError = exports.MissingRequiredSignatureError = exports.NotEnoughTokensError = exports.TokenTransferFailedError = exports.NotEnoughSOLError = exports.MintNotLiveError = exports.MissingCollectionInstructionError = exports.MintNotLastTransactionError = exports.CollectionUpdateAuthorityKeyMismatchError = exports.MissingCollectionAccountsError = exports.CollectionKeyMismatchError = exports.NumericalOverflowErrorError = exports.MissingRemainingAccountError = exports.UninitializedError = exports.IncorrectOwnerError = exports.DataIncrementLimitExceededError = exports.PublicKeyMismatchError = exports.DeserializationErrorError = exports.InvalidAccountSizeError = void 0;
const createErrorFromCodeLookup = new Map();
const createErrorFromNameLookup = new Map();
class InvalidAccountSizeError extends Error {
    constructor() {
        super('Could not save guard to account');
        this.code = 0x1770;
        this.name = 'InvalidAccountSize';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, InvalidAccountSizeError);
        }
    }
}
exports.InvalidAccountSizeError = InvalidAccountSizeError;
createErrorFromCodeLookup.set(0x1770, () => new InvalidAccountSizeError());
createErrorFromNameLookup.set('InvalidAccountSize', () => new InvalidAccountSizeError());
class DeserializationErrorError extends Error {
    constructor() {
        super('Could not deserialize guard');
        this.code = 0x1771;
        this.name = 'DeserializationError';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, DeserializationErrorError);
        }
    }
}
exports.DeserializationErrorError = DeserializationErrorError;
createErrorFromCodeLookup.set(0x1771, () => new DeserializationErrorError());
createErrorFromNameLookup.set('DeserializationError', () => new DeserializationErrorError());
class PublicKeyMismatchError extends Error {
    constructor() {
        super('Public key mismatch');
        this.code = 0x1772;
        this.name = 'PublicKeyMismatch';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, PublicKeyMismatchError);
        }
    }
}
exports.PublicKeyMismatchError = PublicKeyMismatchError;
createErrorFromCodeLookup.set(0x1772, () => new PublicKeyMismatchError());
createErrorFromNameLookup.set('PublicKeyMismatch', () => new PublicKeyMismatchError());
class DataIncrementLimitExceededError extends Error {
    constructor() {
        super('Missing expected remaining account');
        this.code = 0x1773;
        this.name = 'DataIncrementLimitExceeded';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, DataIncrementLimitExceededError);
        }
    }
}
exports.DataIncrementLimitExceededError = DataIncrementLimitExceededError;
createErrorFromCodeLookup.set(0x1773, () => new DataIncrementLimitExceededError());
createErrorFromNameLookup.set('DataIncrementLimitExceeded', () => new DataIncrementLimitExceededError());
class IncorrectOwnerError extends Error {
    constructor() {
        super('Account does not have correct owner');
        this.code = 0x1774;
        this.name = 'IncorrectOwner';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, IncorrectOwnerError);
        }
    }
}
exports.IncorrectOwnerError = IncorrectOwnerError;
createErrorFromCodeLookup.set(0x1774, () => new IncorrectOwnerError());
createErrorFromNameLookup.set('IncorrectOwner', () => new IncorrectOwnerError());
class UninitializedError extends Error {
    constructor() {
        super('Account is not initialized');
        this.code = 0x1775;
        this.name = 'Uninitialized';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, UninitializedError);
        }
    }
}
exports.UninitializedError = UninitializedError;
createErrorFromCodeLookup.set(0x1775, () => new UninitializedError());
createErrorFromNameLookup.set('Uninitialized', () => new UninitializedError());
class MissingRemainingAccountError extends Error {
    constructor() {
        super('Missing expected remaining account');
        this.code = 0x1776;
        this.name = 'MissingRemainingAccount';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MissingRemainingAccountError);
        }
    }
}
exports.MissingRemainingAccountError = MissingRemainingAccountError;
createErrorFromCodeLookup.set(0x1776, () => new MissingRemainingAccountError());
createErrorFromNameLookup.set('MissingRemainingAccount', () => new MissingRemainingAccountError());
class NumericalOverflowErrorError extends Error {
    constructor() {
        super('Numerical overflow error');
        this.code = 0x1777;
        this.name = 'NumericalOverflowError';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, NumericalOverflowErrorError);
        }
    }
}
exports.NumericalOverflowErrorError = NumericalOverflowErrorError;
createErrorFromCodeLookup.set(0x1777, () => new NumericalOverflowErrorError());
createErrorFromNameLookup.set('NumericalOverflowError', () => new NumericalOverflowErrorError());
class CollectionKeyMismatchError extends Error {
    constructor() {
        super('Collection public key mismatch');
        this.code = 0x1778;
        this.name = 'CollectionKeyMismatch';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CollectionKeyMismatchError);
        }
    }
}
exports.CollectionKeyMismatchError = CollectionKeyMismatchError;
createErrorFromCodeLookup.set(0x1778, () => new CollectionKeyMismatchError());
createErrorFromNameLookup.set('CollectionKeyMismatch', () => new CollectionKeyMismatchError());
class MissingCollectionAccountsError extends Error {
    constructor() {
        super('Missing collection accounts');
        this.code = 0x1779;
        this.name = 'MissingCollectionAccounts';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MissingCollectionAccountsError);
        }
    }
}
exports.MissingCollectionAccountsError = MissingCollectionAccountsError;
createErrorFromCodeLookup.set(0x1779, () => new MissingCollectionAccountsError());
createErrorFromNameLookup.set('MissingCollectionAccounts', () => new MissingCollectionAccountsError());
class CollectionUpdateAuthorityKeyMismatchError extends Error {
    constructor() {
        super('Collection update authority public key mismatch');
        this.code = 0x177a;
        this.name = 'CollectionUpdateAuthorityKeyMismatch';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CollectionUpdateAuthorityKeyMismatchError);
        }
    }
}
exports.CollectionUpdateAuthorityKeyMismatchError = CollectionUpdateAuthorityKeyMismatchError;
createErrorFromCodeLookup.set(0x177a, () => new CollectionUpdateAuthorityKeyMismatchError());
createErrorFromNameLookup.set('CollectionUpdateAuthorityKeyMismatch', () => new CollectionUpdateAuthorityKeyMismatchError());
class MintNotLastTransactionError extends Error {
    constructor() {
        super('Mint must be the last instructions of the transaction');
        this.code = 0x177b;
        this.name = 'MintNotLastTransaction';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MintNotLastTransactionError);
        }
    }
}
exports.MintNotLastTransactionError = MintNotLastTransactionError;
createErrorFromCodeLookup.set(0x177b, () => new MintNotLastTransactionError());
createErrorFromNameLookup.set('MintNotLastTransaction', () => new MintNotLastTransactionError());
class MissingCollectionInstructionError extends Error {
    constructor() {
        super('Missing set collection during mint IX');
        this.code = 0x177c;
        this.name = 'MissingCollectionInstruction';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MissingCollectionInstructionError);
        }
    }
}
exports.MissingCollectionInstructionError = MissingCollectionInstructionError;
createErrorFromCodeLookup.set(0x177c, () => new MissingCollectionInstructionError());
createErrorFromNameLookup.set('MissingCollectionInstruction', () => new MissingCollectionInstructionError());
class MintNotLiveError extends Error {
    constructor() {
        super('Mint is not live');
        this.code = 0x177d;
        this.name = 'MintNotLive';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MintNotLiveError);
        }
    }
}
exports.MintNotLiveError = MintNotLiveError;
createErrorFromCodeLookup.set(0x177d, () => new MintNotLiveError());
createErrorFromNameLookup.set('MintNotLive', () => new MintNotLiveError());
class NotEnoughSOLError extends Error {
    constructor() {
        super('Not enough SOL to pay for the mint');
        this.code = 0x177e;
        this.name = 'NotEnoughSOL';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, NotEnoughSOLError);
        }
    }
}
exports.NotEnoughSOLError = NotEnoughSOLError;
createErrorFromCodeLookup.set(0x177e, () => new NotEnoughSOLError());
createErrorFromNameLookup.set('NotEnoughSOL', () => new NotEnoughSOLError());
class TokenTransferFailedError extends Error {
    constructor() {
        super('Token transfer failed');
        this.code = 0x177f;
        this.name = 'TokenTransferFailed';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, TokenTransferFailedError);
        }
    }
}
exports.TokenTransferFailedError = TokenTransferFailedError;
createErrorFromCodeLookup.set(0x177f, () => new TokenTransferFailedError());
createErrorFromNameLookup.set('TokenTransferFailed', () => new TokenTransferFailedError());
class NotEnoughTokensError extends Error {
    constructor() {
        super('Not enough tokens to pay for this minting');
        this.code = 0x1780;
        this.name = 'NotEnoughTokens';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, NotEnoughTokensError);
        }
    }
}
exports.NotEnoughTokensError = NotEnoughTokensError;
createErrorFromCodeLookup.set(0x1780, () => new NotEnoughTokensError());
createErrorFromNameLookup.set('NotEnoughTokens', () => new NotEnoughTokensError());
class MissingRequiredSignatureError extends Error {
    constructor() {
        super('A signature was required but not found');
        this.code = 0x1781;
        this.name = 'MissingRequiredSignature';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MissingRequiredSignatureError);
        }
    }
}
exports.MissingRequiredSignatureError = MissingRequiredSignatureError;
createErrorFromCodeLookup.set(0x1781, () => new MissingRequiredSignatureError());
createErrorFromNameLookup.set('MissingRequiredSignature', () => new MissingRequiredSignatureError());
class TokenBurnFailedError extends Error {
    constructor() {
        super('Token burn failed');
        this.code = 0x1782;
        this.name = 'TokenBurnFailed';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, TokenBurnFailedError);
        }
    }
}
exports.TokenBurnFailedError = TokenBurnFailedError;
createErrorFromCodeLookup.set(0x1782, () => new TokenBurnFailedError());
createErrorFromNameLookup.set('TokenBurnFailed', () => new TokenBurnFailedError());
class NoWhitelistTokenError extends Error {
    constructor() {
        super('No whitelist token present');
        this.code = 0x1783;
        this.name = 'NoWhitelistToken';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, NoWhitelistTokenError);
        }
    }
}
exports.NoWhitelistTokenError = NoWhitelistTokenError;
createErrorFromCodeLookup.set(0x1783, () => new NoWhitelistTokenError());
createErrorFromNameLookup.set('NoWhitelistToken', () => new NoWhitelistTokenError());
class GatewayTokenInvalidError extends Error {
    constructor() {
        super('Gateway token is not valid');
        this.code = 0x1784;
        this.name = 'GatewayTokenInvalid';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, GatewayTokenInvalidError);
        }
    }
}
exports.GatewayTokenInvalidError = GatewayTokenInvalidError;
createErrorFromCodeLookup.set(0x1784, () => new GatewayTokenInvalidError());
createErrorFromNameLookup.set('GatewayTokenInvalid', () => new GatewayTokenInvalidError());
class AfterEndSettingsDateError extends Error {
    constructor() {
        super('Current time is after the set end settings date');
        this.code = 0x1785;
        this.name = 'AfterEndSettingsDate';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, AfterEndSettingsDateError);
        }
    }
}
exports.AfterEndSettingsDateError = AfterEndSettingsDateError;
createErrorFromCodeLookup.set(0x1785, () => new AfterEndSettingsDateError());
createErrorFromNameLookup.set('AfterEndSettingsDate', () => new AfterEndSettingsDateError());
class AfterEndSettingsMintAmountError extends Error {
    constructor() {
        super('Current items minted is at the set end settings amount');
        this.code = 0x1786;
        this.name = 'AfterEndSettingsMintAmount';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, AfterEndSettingsMintAmountError);
        }
    }
}
exports.AfterEndSettingsMintAmountError = AfterEndSettingsMintAmountError;
createErrorFromCodeLookup.set(0x1786, () => new AfterEndSettingsMintAmountError());
createErrorFromNameLookup.set('AfterEndSettingsMintAmount', () => new AfterEndSettingsMintAmountError());
function errorFromCode(code) {
    const createError = createErrorFromCodeLookup.get(code);
    return createError != null ? createError() : null;
}
exports.errorFromCode = errorFromCode;
function errorFromName(name) {
    const createError = createErrorFromNameLookup.get(name);
    return createError != null ? createError() : null;
}
exports.errorFromName = errorFromName;
//# sourceMappingURL=index.js.map