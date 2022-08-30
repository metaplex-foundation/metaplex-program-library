declare type ErrorWithCode = Error & {
    code: number;
};
declare type MaybeErrorWithCode = ErrorWithCode | null | undefined;
export declare class IncorrectOwnerError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class UninitializedError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MintMismatchError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class IndexGreaterThanLengthError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class NumericalOverflowErrorError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class TooManyCreatorsError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CandyMachineEmptyError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class HiddenSettingsDoNotHaveConfigLinesError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CannotChangeNumberOfLinesError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CannotSwitchToHiddenSettingsError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class IncorrectCollectionAuthorityError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MetadataAccountMustBeEmptyError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class NoChangingCollectionDuringMintError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CandyCollectionRequiresRetainAuthorityError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class ExceededLengthErrorError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MissingConfigLinesSettingsError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CannotIncreaseLengthError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CannotSwitchFromHiddenSettingsError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CannotChangeSequentialIndexGenerationError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CollectionKeyMismatchError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MissingCollectionAccountsError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CannotChangeUpdateAuthorityError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare function errorFromCode(code: number): MaybeErrorWithCode;
export declare function errorFromName(name: string): MaybeErrorWithCode;
export {};
