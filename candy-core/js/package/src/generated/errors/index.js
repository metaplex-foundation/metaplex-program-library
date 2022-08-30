"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.errorFromName = exports.errorFromCode = exports.CannotChangeUpdateAuthorityError = exports.MissingCollectionAccountsError = exports.CollectionKeyMismatchError = exports.CannotChangeSequentialIndexGenerationError = exports.CannotSwitchFromHiddenSettingsError = exports.CannotIncreaseLengthError = exports.MissingConfigLinesSettingsError = exports.ExceededLengthErrorError = exports.CandyCollectionRequiresRetainAuthorityError = exports.NoChangingCollectionDuringMintError = exports.MetadataAccountMustBeEmptyError = exports.IncorrectCollectionAuthorityError = exports.CannotSwitchToHiddenSettingsError = exports.CannotChangeNumberOfLinesError = exports.HiddenSettingsDoNotHaveConfigLinesError = exports.CandyMachineEmptyError = exports.TooManyCreatorsError = exports.NumericalOverflowErrorError = exports.IndexGreaterThanLengthError = exports.MintMismatchError = exports.UninitializedError = exports.IncorrectOwnerError = void 0;
const createErrorFromCodeLookup = new Map();
const createErrorFromNameLookup = new Map();
class IncorrectOwnerError extends Error {
    constructor() {
        super('Account does not have correct owner');
        this.code = 0x1770;
        this.name = 'IncorrectOwner';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, IncorrectOwnerError);
        }
    }
}
exports.IncorrectOwnerError = IncorrectOwnerError;
createErrorFromCodeLookup.set(0x1770, () => new IncorrectOwnerError());
createErrorFromNameLookup.set('IncorrectOwner', () => new IncorrectOwnerError());
class UninitializedError extends Error {
    constructor() {
        super('Account is not initialized');
        this.code = 0x1771;
        this.name = 'Uninitialized';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, UninitializedError);
        }
    }
}
exports.UninitializedError = UninitializedError;
createErrorFromCodeLookup.set(0x1771, () => new UninitializedError());
createErrorFromNameLookup.set('Uninitialized', () => new UninitializedError());
class MintMismatchError extends Error {
    constructor() {
        super('Mint Mismatch');
        this.code = 0x1772;
        this.name = 'MintMismatch';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MintMismatchError);
        }
    }
}
exports.MintMismatchError = MintMismatchError;
createErrorFromCodeLookup.set(0x1772, () => new MintMismatchError());
createErrorFromNameLookup.set('MintMismatch', () => new MintMismatchError());
class IndexGreaterThanLengthError extends Error {
    constructor() {
        super('Index greater than length');
        this.code = 0x1773;
        this.name = 'IndexGreaterThanLength';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, IndexGreaterThanLengthError);
        }
    }
}
exports.IndexGreaterThanLengthError = IndexGreaterThanLengthError;
createErrorFromCodeLookup.set(0x1773, () => new IndexGreaterThanLengthError());
createErrorFromNameLookup.set('IndexGreaterThanLength', () => new IndexGreaterThanLengthError());
class NumericalOverflowErrorError extends Error {
    constructor() {
        super('Numerical overflow error');
        this.code = 0x1774;
        this.name = 'NumericalOverflowError';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, NumericalOverflowErrorError);
        }
    }
}
exports.NumericalOverflowErrorError = NumericalOverflowErrorError;
createErrorFromCodeLookup.set(0x1774, () => new NumericalOverflowErrorError());
createErrorFromNameLookup.set('NumericalOverflowError', () => new NumericalOverflowErrorError());
class TooManyCreatorsError extends Error {
    constructor() {
        super('Can only provide up to 4 creators to candy machine (because candy machine is one)');
        this.code = 0x1775;
        this.name = 'TooManyCreators';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, TooManyCreatorsError);
        }
    }
}
exports.TooManyCreatorsError = TooManyCreatorsError;
createErrorFromCodeLookup.set(0x1775, () => new TooManyCreatorsError());
createErrorFromNameLookup.set('TooManyCreators', () => new TooManyCreatorsError());
class CandyMachineEmptyError extends Error {
    constructor() {
        super('Candy machine is empty');
        this.code = 0x1776;
        this.name = 'CandyMachineEmpty';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CandyMachineEmptyError);
        }
    }
}
exports.CandyMachineEmptyError = CandyMachineEmptyError;
createErrorFromCodeLookup.set(0x1776, () => new CandyMachineEmptyError());
createErrorFromNameLookup.set('CandyMachineEmpty', () => new CandyMachineEmptyError());
class HiddenSettingsDoNotHaveConfigLinesError extends Error {
    constructor() {
        super('Candy machines using hidden uris do not have config lines, they have a single hash representing hashed order');
        this.code = 0x1777;
        this.name = 'HiddenSettingsDoNotHaveConfigLines';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, HiddenSettingsDoNotHaveConfigLinesError);
        }
    }
}
exports.HiddenSettingsDoNotHaveConfigLinesError = HiddenSettingsDoNotHaveConfigLinesError;
createErrorFromCodeLookup.set(0x1777, () => new HiddenSettingsDoNotHaveConfigLinesError());
createErrorFromNameLookup.set('HiddenSettingsDoNotHaveConfigLines', () => new HiddenSettingsDoNotHaveConfigLinesError());
class CannotChangeNumberOfLinesError extends Error {
    constructor() {
        super('Cannot change number of lines unless is a hidden config');
        this.code = 0x1778;
        this.name = 'CannotChangeNumberOfLines';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CannotChangeNumberOfLinesError);
        }
    }
}
exports.CannotChangeNumberOfLinesError = CannotChangeNumberOfLinesError;
createErrorFromCodeLookup.set(0x1778, () => new CannotChangeNumberOfLinesError());
createErrorFromNameLookup.set('CannotChangeNumberOfLines', () => new CannotChangeNumberOfLinesError());
class CannotSwitchToHiddenSettingsError extends Error {
    constructor() {
        super('Cannot switch to hidden settings after items available is greater than 0');
        this.code = 0x1779;
        this.name = 'CannotSwitchToHiddenSettings';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CannotSwitchToHiddenSettingsError);
        }
    }
}
exports.CannotSwitchToHiddenSettingsError = CannotSwitchToHiddenSettingsError;
createErrorFromCodeLookup.set(0x1779, () => new CannotSwitchToHiddenSettingsError());
createErrorFromNameLookup.set('CannotSwitchToHiddenSettings', () => new CannotSwitchToHiddenSettingsError());
class IncorrectCollectionAuthorityError extends Error {
    constructor() {
        super('Incorrect collection NFT authority');
        this.code = 0x177a;
        this.name = 'IncorrectCollectionAuthority';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, IncorrectCollectionAuthorityError);
        }
    }
}
exports.IncorrectCollectionAuthorityError = IncorrectCollectionAuthorityError;
createErrorFromCodeLookup.set(0x177a, () => new IncorrectCollectionAuthorityError());
createErrorFromNameLookup.set('IncorrectCollectionAuthority', () => new IncorrectCollectionAuthorityError());
class MetadataAccountMustBeEmptyError extends Error {
    constructor() {
        super('The metadata account has data in it, and this must be empty to mint a new NFT');
        this.code = 0x177b;
        this.name = 'MetadataAccountMustBeEmpty';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MetadataAccountMustBeEmptyError);
        }
    }
}
exports.MetadataAccountMustBeEmptyError = MetadataAccountMustBeEmptyError;
createErrorFromCodeLookup.set(0x177b, () => new MetadataAccountMustBeEmptyError());
createErrorFromNameLookup.set('MetadataAccountMustBeEmpty', () => new MetadataAccountMustBeEmptyError());
class NoChangingCollectionDuringMintError extends Error {
    constructor() {
        super("Can't change collection settings after items have begun to be minted");
        this.code = 0x177c;
        this.name = 'NoChangingCollectionDuringMint';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, NoChangingCollectionDuringMintError);
        }
    }
}
exports.NoChangingCollectionDuringMintError = NoChangingCollectionDuringMintError;
createErrorFromCodeLookup.set(0x177c, () => new NoChangingCollectionDuringMintError());
createErrorFromNameLookup.set('NoChangingCollectionDuringMint', () => new NoChangingCollectionDuringMintError());
class CandyCollectionRequiresRetainAuthorityError extends Error {
    constructor() {
        super('Retain authority must be true for Candy Machines with a collection set');
        this.code = 0x177d;
        this.name = 'CandyCollectionRequiresRetainAuthority';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CandyCollectionRequiresRetainAuthorityError);
        }
    }
}
exports.CandyCollectionRequiresRetainAuthorityError = CandyCollectionRequiresRetainAuthorityError;
createErrorFromCodeLookup.set(0x177d, () => new CandyCollectionRequiresRetainAuthorityError());
createErrorFromNameLookup.set('CandyCollectionRequiresRetainAuthority', () => new CandyCollectionRequiresRetainAuthorityError());
class ExceededLengthErrorError extends Error {
    constructor() {
        super('Value longer than expected maximum value');
        this.code = 0x177e;
        this.name = 'ExceededLengthError';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, ExceededLengthErrorError);
        }
    }
}
exports.ExceededLengthErrorError = ExceededLengthErrorError;
createErrorFromCodeLookup.set(0x177e, () => new ExceededLengthErrorError());
createErrorFromNameLookup.set('ExceededLengthError', () => new ExceededLengthErrorError());
class MissingConfigLinesSettingsError extends Error {
    constructor() {
        super('Missing config lines settings');
        this.code = 0x177f;
        this.name = 'MissingConfigLinesSettings';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MissingConfigLinesSettingsError);
        }
    }
}
exports.MissingConfigLinesSettingsError = MissingConfigLinesSettingsError;
createErrorFromCodeLookup.set(0x177f, () => new MissingConfigLinesSettingsError());
createErrorFromNameLookup.set('MissingConfigLinesSettings', () => new MissingConfigLinesSettingsError());
class CannotIncreaseLengthError extends Error {
    constructor() {
        super('Cannot increase the length in config lines settings');
        this.code = 0x1780;
        this.name = 'CannotIncreaseLength';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CannotIncreaseLengthError);
        }
    }
}
exports.CannotIncreaseLengthError = CannotIncreaseLengthError;
createErrorFromCodeLookup.set(0x1780, () => new CannotIncreaseLengthError());
createErrorFromNameLookup.set('CannotIncreaseLength', () => new CannotIncreaseLengthError());
class CannotSwitchFromHiddenSettingsError extends Error {
    constructor() {
        super('Cannot switch from hidden settings');
        this.code = 0x1781;
        this.name = 'CannotSwitchFromHiddenSettings';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CannotSwitchFromHiddenSettingsError);
        }
    }
}
exports.CannotSwitchFromHiddenSettingsError = CannotSwitchFromHiddenSettingsError;
createErrorFromCodeLookup.set(0x1781, () => new CannotSwitchFromHiddenSettingsError());
createErrorFromNameLookup.set('CannotSwitchFromHiddenSettings', () => new CannotSwitchFromHiddenSettingsError());
class CannotChangeSequentialIndexGenerationError extends Error {
    constructor() {
        super('Cannot change sequential index generation after items have begun to be minted');
        this.code = 0x1782;
        this.name = 'CannotChangeSequentialIndexGeneration';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CannotChangeSequentialIndexGenerationError);
        }
    }
}
exports.CannotChangeSequentialIndexGenerationError = CannotChangeSequentialIndexGenerationError;
createErrorFromCodeLookup.set(0x1782, () => new CannotChangeSequentialIndexGenerationError());
createErrorFromNameLookup.set('CannotChangeSequentialIndexGeneration', () => new CannotChangeSequentialIndexGenerationError());
class CollectionKeyMismatchError extends Error {
    constructor() {
        super('Collection public key mismatch');
        this.code = 0x1783;
        this.name = 'CollectionKeyMismatch';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CollectionKeyMismatchError);
        }
    }
}
exports.CollectionKeyMismatchError = CollectionKeyMismatchError;
createErrorFromCodeLookup.set(0x1783, () => new CollectionKeyMismatchError());
createErrorFromNameLookup.set('CollectionKeyMismatch', () => new CollectionKeyMismatchError());
class MissingCollectionAccountsError extends Error {
    constructor() {
        super('Missing collection accounts');
        this.code = 0x1784;
        this.name = 'MissingCollectionAccounts';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, MissingCollectionAccountsError);
        }
    }
}
exports.MissingCollectionAccountsError = MissingCollectionAccountsError;
createErrorFromCodeLookup.set(0x1784, () => new MissingCollectionAccountsError());
createErrorFromNameLookup.set('MissingCollectionAccounts', () => new MissingCollectionAccountsError());
class CannotChangeUpdateAuthorityError extends Error {
    constructor() {
        super('Cannot change update authority if a collection mint is set');
        this.code = 0x1785;
        this.name = 'CannotChangeUpdateAuthority';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(this, CannotChangeUpdateAuthorityError);
        }
    }
}
exports.CannotChangeUpdateAuthorityError = CannotChangeUpdateAuthorityError;
createErrorFromCodeLookup.set(0x1785, () => new CannotChangeUpdateAuthorityError());
createErrorFromNameLookup.set('CannotChangeUpdateAuthority', () => new CannotChangeUpdateAuthorityError());
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