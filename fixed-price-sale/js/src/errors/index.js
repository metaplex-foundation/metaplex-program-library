"use strict";
var __extends = (this && this.__extends) || (function () {
    var extendStatics = function (d, b) {
        extendStatics = Object.setPrototypeOf ||
            ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
            function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };
        return extendStatics(d, b);
    };
    return function (d, b) {
        if (typeof b !== "function" && b !== null)
            throw new TypeError("Class extends value " + String(b) + " is not a constructor or null");
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.errorFromName = exports.errorFromCode = exports.CreatorsIsEmptyError = exports.CreatorsIsGtThanAvailableError = exports.PrimarySaleIsNotAllowedError = exports.MetadataShouldBeMutableError = exports.UserWalletMustMatchUserTokenAccountError = exports.MetadataCreatorsIsEmptyError = exports.SellingResourceInInvalidStateError = exports.TreasuryIsNotEmptyError = exports.InvalidFunderDestinationError = exports.PayoutTicketExistsError = exports.FunderIsInvalidError = exports.PriceIsZeroError = exports.MarketInInvalidStateError = exports.MarketIsImmutableError = exports.MarketIsSuspendedError = exports.MarketDurationIsNotUnlimitedError = exports.SupplyIsGtThanMaxSupplyError = exports.MathOverflowError = exports.UserReachBuyLimitError = exports.MarketIsEndedError = exports.MarketIsNotStartedError = exports.IncorrectOwnerError = exports.EndDateIsEarlierThanBeginDateError = exports.StartDateIsInPastError = exports.PiecesInOneWalletIsTooMuchError = exports.PublicKeyMismatchError = exports.SellingResourceOwnerInvalidError = exports.DerivedKeyInvalidError = exports.SupplyIsNotProvidedError = exports.SupplyIsGtThanAvailableError = exports.DescriptionIsTooLongError = exports.NameIsTooLongError = exports.StringIsTooLongError = exports.NoValidSignerPresentError = void 0;
var createErrorFromCodeLookup = new Map();
var createErrorFromNameLookup = new Map();
/**
 * NoValidSignerPresent: 'No valid signer present'
 */
var NoValidSignerPresentError = /** @class */ (function (_super) {
    __extends(NoValidSignerPresentError, _super);
    function NoValidSignerPresentError() {
        var _this = _super.call(this, 'No valid signer present') || this;
        _this.code = 0x1770;
        _this.name = 'NoValidSignerPresent';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, NoValidSignerPresentError);
        }
        return _this;
    }
    return NoValidSignerPresentError;
}(Error));
exports.NoValidSignerPresentError = NoValidSignerPresentError;
createErrorFromCodeLookup.set(0x1770, function () { return new NoValidSignerPresentError(); });
createErrorFromNameLookup.set('NoValidSignerPresent', function () { return new NoValidSignerPresentError(); });
/**
 * StringIsTooLong: 'Some string variable is longer than allowed'
 */
var StringIsTooLongError = /** @class */ (function (_super) {
    __extends(StringIsTooLongError, _super);
    function StringIsTooLongError() {
        var _this = _super.call(this, 'Some string variable is longer than allowed') || this;
        _this.code = 0x1771;
        _this.name = 'StringIsTooLong';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, StringIsTooLongError);
        }
        return _this;
    }
    return StringIsTooLongError;
}(Error));
exports.StringIsTooLongError = StringIsTooLongError;
createErrorFromCodeLookup.set(0x1771, function () { return new StringIsTooLongError(); });
createErrorFromNameLookup.set('StringIsTooLong', function () { return new StringIsTooLongError(); });
/**
 * NameIsTooLong: 'Name string variable is longer than allowed'
 */
var NameIsTooLongError = /** @class */ (function (_super) {
    __extends(NameIsTooLongError, _super);
    function NameIsTooLongError() {
        var _this = _super.call(this, 'Name string variable is longer than allowed') || this;
        _this.code = 0x1772;
        _this.name = 'NameIsTooLong';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, NameIsTooLongError);
        }
        return _this;
    }
    return NameIsTooLongError;
}(Error));
exports.NameIsTooLongError = NameIsTooLongError;
createErrorFromCodeLookup.set(0x1772, function () { return new NameIsTooLongError(); });
createErrorFromNameLookup.set('NameIsTooLong', function () { return new NameIsTooLongError(); });
/**
 * DescriptionIsTooLong: 'Description string variable is longer than allowed'
 */
var DescriptionIsTooLongError = /** @class */ (function (_super) {
    __extends(DescriptionIsTooLongError, _super);
    function DescriptionIsTooLongError() {
        var _this = _super.call(this, 'Description string variable is longer than allowed') || this;
        _this.code = 0x1773;
        _this.name = 'DescriptionIsTooLong';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, DescriptionIsTooLongError);
        }
        return _this;
    }
    return DescriptionIsTooLongError;
}(Error));
exports.DescriptionIsTooLongError = DescriptionIsTooLongError;
createErrorFromCodeLookup.set(0x1773, function () { return new DescriptionIsTooLongError(); });
createErrorFromNameLookup.set('DescriptionIsTooLong', function () { return new DescriptionIsTooLongError(); });
/**
 * SupplyIsGtThanAvailable: 'Provided supply is gt than available'
 */
var SupplyIsGtThanAvailableError = /** @class */ (function (_super) {
    __extends(SupplyIsGtThanAvailableError, _super);
    function SupplyIsGtThanAvailableError() {
        var _this = _super.call(this, 'Provided supply is gt than available') || this;
        _this.code = 0x1774;
        _this.name = 'SupplyIsGtThanAvailable';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, SupplyIsGtThanAvailableError);
        }
        return _this;
    }
    return SupplyIsGtThanAvailableError;
}(Error));
exports.SupplyIsGtThanAvailableError = SupplyIsGtThanAvailableError;
createErrorFromCodeLookup.set(0x1774, function () { return new SupplyIsGtThanAvailableError(); });
createErrorFromNameLookup.set('SupplyIsGtThanAvailable', function () { return new SupplyIsGtThanAvailableError(); });
/**
 * SupplyIsNotProvided: 'Supply is not provided'
 */
var SupplyIsNotProvidedError = /** @class */ (function (_super) {
    __extends(SupplyIsNotProvidedError, _super);
    function SupplyIsNotProvidedError() {
        var _this = _super.call(this, 'Supply is not provided') || this;
        _this.code = 0x1775;
        _this.name = 'SupplyIsNotProvided';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, SupplyIsNotProvidedError);
        }
        return _this;
    }
    return SupplyIsNotProvidedError;
}(Error));
exports.SupplyIsNotProvidedError = SupplyIsNotProvidedError;
createErrorFromCodeLookup.set(0x1775, function () { return new SupplyIsNotProvidedError(); });
createErrorFromNameLookup.set('SupplyIsNotProvided', function () { return new SupplyIsNotProvidedError(); });
/**
 * DerivedKeyInvalid: 'Derived key invalid'
 */
var DerivedKeyInvalidError = /** @class */ (function (_super) {
    __extends(DerivedKeyInvalidError, _super);
    function DerivedKeyInvalidError() {
        var _this = _super.call(this, 'Derived key invalid') || this;
        _this.code = 0x1776;
        _this.name = 'DerivedKeyInvalid';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, DerivedKeyInvalidError);
        }
        return _this;
    }
    return DerivedKeyInvalidError;
}(Error));
exports.DerivedKeyInvalidError = DerivedKeyInvalidError;
createErrorFromCodeLookup.set(0x1776, function () { return new DerivedKeyInvalidError(); });
createErrorFromNameLookup.set('DerivedKeyInvalid', function () { return new DerivedKeyInvalidError(); });
/**
 * SellingResourceOwnerInvalid: 'Invalid selling resource owner provided'
 */
var SellingResourceOwnerInvalidError = /** @class */ (function (_super) {
    __extends(SellingResourceOwnerInvalidError, _super);
    function SellingResourceOwnerInvalidError() {
        var _this = _super.call(this, 'Invalid selling resource owner provided') || this;
        _this.code = 0x1777;
        _this.name = 'SellingResourceOwnerInvalid';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, SellingResourceOwnerInvalidError);
        }
        return _this;
    }
    return SellingResourceOwnerInvalidError;
}(Error));
exports.SellingResourceOwnerInvalidError = SellingResourceOwnerInvalidError;
createErrorFromCodeLookup.set(0x1777, function () { return new SellingResourceOwnerInvalidError(); });
createErrorFromNameLookup.set('SellingResourceOwnerInvalid', function () { return new SellingResourceOwnerInvalidError(); });
/**
 * PublicKeyMismatch: 'PublicKeyMismatch'
 */
var PublicKeyMismatchError = /** @class */ (function (_super) {
    __extends(PublicKeyMismatchError, _super);
    function PublicKeyMismatchError() {
        var _this = _super.call(this, 'PublicKeyMismatch') || this;
        _this.code = 0x1778;
        _this.name = 'PublicKeyMismatch';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, PublicKeyMismatchError);
        }
        return _this;
    }
    return PublicKeyMismatchError;
}(Error));
exports.PublicKeyMismatchError = PublicKeyMismatchError;
createErrorFromCodeLookup.set(0x1778, function () { return new PublicKeyMismatchError(); });
createErrorFromNameLookup.set('PublicKeyMismatch', function () { return new PublicKeyMismatchError(); });
/**
 * PiecesInOneWalletIsTooMuch: 'Pieces in one wallet cannot be greater than Max Supply value'
 */
var PiecesInOneWalletIsTooMuchError = /** @class */ (function (_super) {
    __extends(PiecesInOneWalletIsTooMuchError, _super);
    function PiecesInOneWalletIsTooMuchError() {
        var _this = _super.call(this, 'Pieces in one wallet cannot be greater than Max Supply value') || this;
        _this.code = 0x1779;
        _this.name = 'PiecesInOneWalletIsTooMuch';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, PiecesInOneWalletIsTooMuchError);
        }
        return _this;
    }
    return PiecesInOneWalletIsTooMuchError;
}(Error));
exports.PiecesInOneWalletIsTooMuchError = PiecesInOneWalletIsTooMuchError;
createErrorFromCodeLookup.set(0x1779, function () { return new PiecesInOneWalletIsTooMuchError(); });
createErrorFromNameLookup.set('PiecesInOneWalletIsTooMuch', function () { return new PiecesInOneWalletIsTooMuchError(); });
/**
 * StartDateIsInPast: 'StartDate cannot be in the past'
 */
var StartDateIsInPastError = /** @class */ (function (_super) {
    __extends(StartDateIsInPastError, _super);
    function StartDateIsInPastError() {
        var _this = _super.call(this, 'StartDate cannot be in the past') || this;
        _this.code = 0x177a;
        _this.name = 'StartDateIsInPast';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, StartDateIsInPastError);
        }
        return _this;
    }
    return StartDateIsInPastError;
}(Error));
exports.StartDateIsInPastError = StartDateIsInPastError;
createErrorFromCodeLookup.set(0x177a, function () { return new StartDateIsInPastError(); });
createErrorFromNameLookup.set('StartDateIsInPast', function () { return new StartDateIsInPastError(); });
/**
 * EndDateIsEarlierThanBeginDate: 'EndDate should not be earlier than StartDate'
 */
var EndDateIsEarlierThanBeginDateError = /** @class */ (function (_super) {
    __extends(EndDateIsEarlierThanBeginDateError, _super);
    function EndDateIsEarlierThanBeginDateError() {
        var _this = _super.call(this, 'EndDate should not be earlier than StartDate') || this;
        _this.code = 0x177b;
        _this.name = 'EndDateIsEarlierThanBeginDate';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, EndDateIsEarlierThanBeginDateError);
        }
        return _this;
    }
    return EndDateIsEarlierThanBeginDateError;
}(Error));
exports.EndDateIsEarlierThanBeginDateError = EndDateIsEarlierThanBeginDateError;
createErrorFromCodeLookup.set(0x177b, function () { return new EndDateIsEarlierThanBeginDateError(); });
createErrorFromNameLookup.set('EndDateIsEarlierThanBeginDate', function () { return new EndDateIsEarlierThanBeginDateError(); });
/**
 * IncorrectOwner: 'Incorrect account owner'
 */
var IncorrectOwnerError = /** @class */ (function (_super) {
    __extends(IncorrectOwnerError, _super);
    function IncorrectOwnerError() {
        var _this = _super.call(this, 'Incorrect account owner') || this;
        _this.code = 0x177c;
        _this.name = 'IncorrectOwner';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, IncorrectOwnerError);
        }
        return _this;
    }
    return IncorrectOwnerError;
}(Error));
exports.IncorrectOwnerError = IncorrectOwnerError;
createErrorFromCodeLookup.set(0x177c, function () { return new IncorrectOwnerError(); });
createErrorFromNameLookup.set('IncorrectOwner', function () { return new IncorrectOwnerError(); });
/**
 * MarketIsNotStarted: 'Market is not started'
 */
var MarketIsNotStartedError = /** @class */ (function (_super) {
    __extends(MarketIsNotStartedError, _super);
    function MarketIsNotStartedError() {
        var _this = _super.call(this, 'Market is not started') || this;
        _this.code = 0x177d;
        _this.name = 'MarketIsNotStarted';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, MarketIsNotStartedError);
        }
        return _this;
    }
    return MarketIsNotStartedError;
}(Error));
exports.MarketIsNotStartedError = MarketIsNotStartedError;
createErrorFromCodeLookup.set(0x177d, function () { return new MarketIsNotStartedError(); });
createErrorFromNameLookup.set('MarketIsNotStarted', function () { return new MarketIsNotStartedError(); });
/**
 * MarketIsEnded: 'Market is ended'
 */
var MarketIsEndedError = /** @class */ (function (_super) {
    __extends(MarketIsEndedError, _super);
    function MarketIsEndedError() {
        var _this = _super.call(this, 'Market is ended') || this;
        _this.code = 0x177e;
        _this.name = 'MarketIsEnded';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, MarketIsEndedError);
        }
        return _this;
    }
    return MarketIsEndedError;
}(Error));
exports.MarketIsEndedError = MarketIsEndedError;
createErrorFromCodeLookup.set(0x177e, function () { return new MarketIsEndedError(); });
createErrorFromNameLookup.set('MarketIsEnded', function () { return new MarketIsEndedError(); });
/**
 * UserReachBuyLimit: 'User reach buy limit'
 */
var UserReachBuyLimitError = /** @class */ (function (_super) {
    __extends(UserReachBuyLimitError, _super);
    function UserReachBuyLimitError() {
        var _this = _super.call(this, 'User reach buy limit') || this;
        _this.code = 0x177f;
        _this.name = 'UserReachBuyLimit';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, UserReachBuyLimitError);
        }
        return _this;
    }
    return UserReachBuyLimitError;
}(Error));
exports.UserReachBuyLimitError = UserReachBuyLimitError;
createErrorFromCodeLookup.set(0x177f, function () { return new UserReachBuyLimitError(); });
createErrorFromNameLookup.set('UserReachBuyLimit', function () { return new UserReachBuyLimitError(); });
/**
 * MathOverflow: 'Math overflow'
 */
var MathOverflowError = /** @class */ (function (_super) {
    __extends(MathOverflowError, _super);
    function MathOverflowError() {
        var _this = _super.call(this, 'Math overflow') || this;
        _this.code = 0x1780;
        _this.name = 'MathOverflow';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, MathOverflowError);
        }
        return _this;
    }
    return MathOverflowError;
}(Error));
exports.MathOverflowError = MathOverflowError;
createErrorFromCodeLookup.set(0x1780, function () { return new MathOverflowError(); });
createErrorFromNameLookup.set('MathOverflow', function () { return new MathOverflowError(); });
/**
 * SupplyIsGtThanMaxSupply: 'Supply is gt than max supply'
 */
var SupplyIsGtThanMaxSupplyError = /** @class */ (function (_super) {
    __extends(SupplyIsGtThanMaxSupplyError, _super);
    function SupplyIsGtThanMaxSupplyError() {
        var _this = _super.call(this, 'Supply is gt than max supply') || this;
        _this.code = 0x1781;
        _this.name = 'SupplyIsGtThanMaxSupply';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, SupplyIsGtThanMaxSupplyError);
        }
        return _this;
    }
    return SupplyIsGtThanMaxSupplyError;
}(Error));
exports.SupplyIsGtThanMaxSupplyError = SupplyIsGtThanMaxSupplyError;
createErrorFromCodeLookup.set(0x1781, function () { return new SupplyIsGtThanMaxSupplyError(); });
createErrorFromNameLookup.set('SupplyIsGtThanMaxSupply', function () { return new SupplyIsGtThanMaxSupplyError(); });
/**
 * MarketDurationIsNotUnlimited: 'Market duration is not unlimited'
 */
var MarketDurationIsNotUnlimitedError = /** @class */ (function (_super) {
    __extends(MarketDurationIsNotUnlimitedError, _super);
    function MarketDurationIsNotUnlimitedError() {
        var _this = _super.call(this, 'Market duration is not unlimited') || this;
        _this.code = 0x1782;
        _this.name = 'MarketDurationIsNotUnlimited';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, MarketDurationIsNotUnlimitedError);
        }
        return _this;
    }
    return MarketDurationIsNotUnlimitedError;
}(Error));
exports.MarketDurationIsNotUnlimitedError = MarketDurationIsNotUnlimitedError;
createErrorFromCodeLookup.set(0x1782, function () { return new MarketDurationIsNotUnlimitedError(); });
createErrorFromNameLookup.set('MarketDurationIsNotUnlimited', function () { return new MarketDurationIsNotUnlimitedError(); });
/**
 * MarketIsSuspended: 'Market is suspended'
 */
var MarketIsSuspendedError = /** @class */ (function (_super) {
    __extends(MarketIsSuspendedError, _super);
    function MarketIsSuspendedError() {
        var _this = _super.call(this, 'Market is suspended') || this;
        _this.code = 0x1783;
        _this.name = 'MarketIsSuspended';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, MarketIsSuspendedError);
        }
        return _this;
    }
    return MarketIsSuspendedError;
}(Error));
exports.MarketIsSuspendedError = MarketIsSuspendedError;
createErrorFromCodeLookup.set(0x1783, function () { return new MarketIsSuspendedError(); });
createErrorFromNameLookup.set('MarketIsSuspended', function () { return new MarketIsSuspendedError(); });
/**
 * MarketIsImmutable: 'Market is immutable'
 */
var MarketIsImmutableError = /** @class */ (function (_super) {
    __extends(MarketIsImmutableError, _super);
    function MarketIsImmutableError() {
        var _this = _super.call(this, 'Market is immutable') || this;
        _this.code = 0x1784;
        _this.name = 'MarketIsImmutable';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, MarketIsImmutableError);
        }
        return _this;
    }
    return MarketIsImmutableError;
}(Error));
exports.MarketIsImmutableError = MarketIsImmutableError;
createErrorFromCodeLookup.set(0x1784, function () { return new MarketIsImmutableError(); });
createErrorFromNameLookup.set('MarketIsImmutable', function () { return new MarketIsImmutableError(); });
/**
 * MarketInInvalidState: 'Market in invalid state'
 */
var MarketInInvalidStateError = /** @class */ (function (_super) {
    __extends(MarketInInvalidStateError, _super);
    function MarketInInvalidStateError() {
        var _this = _super.call(this, 'Market in invalid state') || this;
        _this.code = 0x1785;
        _this.name = 'MarketInInvalidState';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, MarketInInvalidStateError);
        }
        return _this;
    }
    return MarketInInvalidStateError;
}(Error));
exports.MarketInInvalidStateError = MarketInInvalidStateError;
createErrorFromCodeLookup.set(0x1785, function () { return new MarketInInvalidStateError(); });
createErrorFromNameLookup.set('MarketInInvalidState', function () { return new MarketInInvalidStateError(); });
/**
 * PriceIsZero: 'Price is zero'
 */
var PriceIsZeroError = /** @class */ (function (_super) {
    __extends(PriceIsZeroError, _super);
    function PriceIsZeroError() {
        var _this = _super.call(this, 'Price is zero') || this;
        _this.code = 0x1786;
        _this.name = 'PriceIsZero';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, PriceIsZeroError);
        }
        return _this;
    }
    return PriceIsZeroError;
}(Error));
exports.PriceIsZeroError = PriceIsZeroError;
createErrorFromCodeLookup.set(0x1786, function () { return new PriceIsZeroError(); });
createErrorFromNameLookup.set('PriceIsZero', function () { return new PriceIsZeroError(); });
/**
 * FunderIsInvalid: 'Funder is invalid'
 */
var FunderIsInvalidError = /** @class */ (function (_super) {
    __extends(FunderIsInvalidError, _super);
    function FunderIsInvalidError() {
        var _this = _super.call(this, 'Funder is invalid') || this;
        _this.code = 0x1787;
        _this.name = 'FunderIsInvalid';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, FunderIsInvalidError);
        }
        return _this;
    }
    return FunderIsInvalidError;
}(Error));
exports.FunderIsInvalidError = FunderIsInvalidError;
createErrorFromCodeLookup.set(0x1787, function () { return new FunderIsInvalidError(); });
createErrorFromNameLookup.set('FunderIsInvalid', function () { return new FunderIsInvalidError(); });
/**
 * PayoutTicketExists: 'Payout ticket exists'
 */
var PayoutTicketExistsError = /** @class */ (function (_super) {
    __extends(PayoutTicketExistsError, _super);
    function PayoutTicketExistsError() {
        var _this = _super.call(this, 'Payout ticket exists') || this;
        _this.code = 0x1788;
        _this.name = 'PayoutTicketExists';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, PayoutTicketExistsError);
        }
        return _this;
    }
    return PayoutTicketExistsError;
}(Error));
exports.PayoutTicketExistsError = PayoutTicketExistsError;
createErrorFromCodeLookup.set(0x1788, function () { return new PayoutTicketExistsError(); });
createErrorFromNameLookup.set('PayoutTicketExists', function () { return new PayoutTicketExistsError(); });
/**
 * InvalidFunderDestination: 'Funder provide invalid destination'
 */
var InvalidFunderDestinationError = /** @class */ (function (_super) {
    __extends(InvalidFunderDestinationError, _super);
    function InvalidFunderDestinationError() {
        var _this = _super.call(this, 'Funder provide invalid destination') || this;
        _this.code = 0x1789;
        _this.name = 'InvalidFunderDestination';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, InvalidFunderDestinationError);
        }
        return _this;
    }
    return InvalidFunderDestinationError;
}(Error));
exports.InvalidFunderDestinationError = InvalidFunderDestinationError;
createErrorFromCodeLookup.set(0x1789, function () { return new InvalidFunderDestinationError(); });
createErrorFromNameLookup.set('InvalidFunderDestination', function () { return new InvalidFunderDestinationError(); });
/**
 * TreasuryIsNotEmpty: 'Treasury is not empty'
 */
var TreasuryIsNotEmptyError = /** @class */ (function (_super) {
    __extends(TreasuryIsNotEmptyError, _super);
    function TreasuryIsNotEmptyError() {
        var _this = _super.call(this, 'Treasury is not empty') || this;
        _this.code = 0x178a;
        _this.name = 'TreasuryIsNotEmpty';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, TreasuryIsNotEmptyError);
        }
        return _this;
    }
    return TreasuryIsNotEmptyError;
}(Error));
exports.TreasuryIsNotEmptyError = TreasuryIsNotEmptyError;
createErrorFromCodeLookup.set(0x178a, function () { return new TreasuryIsNotEmptyError(); });
createErrorFromNameLookup.set('TreasuryIsNotEmpty', function () { return new TreasuryIsNotEmptyError(); });
/**
 * SellingResourceInInvalidState: 'Selling resource in invalid state'
 */
var SellingResourceInInvalidStateError = /** @class */ (function (_super) {
    __extends(SellingResourceInInvalidStateError, _super);
    function SellingResourceInInvalidStateError() {
        var _this = _super.call(this, 'Selling resource in invalid state') || this;
        _this.code = 0x178b;
        _this.name = 'SellingResourceInInvalidState';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, SellingResourceInInvalidStateError);
        }
        return _this;
    }
    return SellingResourceInInvalidStateError;
}(Error));
exports.SellingResourceInInvalidStateError = SellingResourceInInvalidStateError;
createErrorFromCodeLookup.set(0x178b, function () { return new SellingResourceInInvalidStateError(); });
createErrorFromNameLookup.set('SellingResourceInInvalidState', function () { return new SellingResourceInInvalidStateError(); });
/**
 * MetadataCreatorsIsEmpty: 'Metadata creators is empty'
 */
var MetadataCreatorsIsEmptyError = /** @class */ (function (_super) {
    __extends(MetadataCreatorsIsEmptyError, _super);
    function MetadataCreatorsIsEmptyError() {
        var _this = _super.call(this, 'Metadata creators is empty') || this;
        _this.code = 0x178c;
        _this.name = 'MetadataCreatorsIsEmpty';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, MetadataCreatorsIsEmptyError);
        }
        return _this;
    }
    return MetadataCreatorsIsEmptyError;
}(Error));
exports.MetadataCreatorsIsEmptyError = MetadataCreatorsIsEmptyError;
createErrorFromCodeLookup.set(0x178c, function () { return new MetadataCreatorsIsEmptyError(); });
createErrorFromNameLookup.set('MetadataCreatorsIsEmpty', function () { return new MetadataCreatorsIsEmptyError(); });
/**
 * UserWalletMustMatchUserTokenAccount: 'User wallet must match user token account'
 */
var UserWalletMustMatchUserTokenAccountError = /** @class */ (function (_super) {
    __extends(UserWalletMustMatchUserTokenAccountError, _super);
    function UserWalletMustMatchUserTokenAccountError() {
        var _this = _super.call(this, 'User wallet must match user token account') || this;
        _this.code = 0x178d;
        _this.name = 'UserWalletMustMatchUserTokenAccount';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, UserWalletMustMatchUserTokenAccountError);
        }
        return _this;
    }
    return UserWalletMustMatchUserTokenAccountError;
}(Error));
exports.UserWalletMustMatchUserTokenAccountError = UserWalletMustMatchUserTokenAccountError;
createErrorFromCodeLookup.set(0x178d, function () { return new UserWalletMustMatchUserTokenAccountError(); });
createErrorFromNameLookup.set('UserWalletMustMatchUserTokenAccount', function () { return new UserWalletMustMatchUserTokenAccountError(); });
/**
 * MetadataShouldBeMutable: 'Metadata should be mutable'
 */
var MetadataShouldBeMutableError = /** @class */ (function (_super) {
    __extends(MetadataShouldBeMutableError, _super);
    function MetadataShouldBeMutableError() {
        var _this = _super.call(this, 'Metadata should be mutable') || this;
        _this.code = 0x178e;
        _this.name = 'MetadataShouldBeMutable';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, MetadataShouldBeMutableError);
        }
        return _this;
    }
    return MetadataShouldBeMutableError;
}(Error));
exports.MetadataShouldBeMutableError = MetadataShouldBeMutableError;
createErrorFromCodeLookup.set(0x178e, function () { return new MetadataShouldBeMutableError(); });
createErrorFromNameLookup.set('MetadataShouldBeMutable', function () { return new MetadataShouldBeMutableError(); });
/**
 * PrimarySaleIsNotAllowed: 'Primary sale is not allowed'
 */
var PrimarySaleIsNotAllowedError = /** @class */ (function (_super) {
    __extends(PrimarySaleIsNotAllowedError, _super);
    function PrimarySaleIsNotAllowedError() {
        var _this = _super.call(this, 'Primary sale is not allowed') || this;
        _this.code = 0x178f;
        _this.name = 'PrimarySaleIsNotAllowed';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, PrimarySaleIsNotAllowedError);
        }
        return _this;
    }
    return PrimarySaleIsNotAllowedError;
}(Error));
exports.PrimarySaleIsNotAllowedError = PrimarySaleIsNotAllowedError;
createErrorFromCodeLookup.set(0x178f, function () { return new PrimarySaleIsNotAllowedError(); });
createErrorFromNameLookup.set('PrimarySaleIsNotAllowed', function () { return new PrimarySaleIsNotAllowedError(); });
/**
 * CreatorsIsGtThanAvailable: 'Creators is gt than allowed'
 */
var CreatorsIsGtThanAvailableError = /** @class */ (function (_super) {
    __extends(CreatorsIsGtThanAvailableError, _super);
    function CreatorsIsGtThanAvailableError() {
        var _this = _super.call(this, 'Creators is gt than allowed') || this;
        _this.code = 0x1790;
        _this.name = 'CreatorsIsGtThanAvailable';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, CreatorsIsGtThanAvailableError);
        }
        return _this;
    }
    return CreatorsIsGtThanAvailableError;
}(Error));
exports.CreatorsIsGtThanAvailableError = CreatorsIsGtThanAvailableError;
createErrorFromCodeLookup.set(0x1790, function () { return new CreatorsIsGtThanAvailableError(); });
createErrorFromNameLookup.set('CreatorsIsGtThanAvailable', function () { return new CreatorsIsGtThanAvailableError(); });
/**
 * CreatorsIsEmpty: 'Creators is empty'
 */
var CreatorsIsEmptyError = /** @class */ (function (_super) {
    __extends(CreatorsIsEmptyError, _super);
    function CreatorsIsEmptyError() {
        var _this = _super.call(this, 'Creators is empty') || this;
        _this.code = 0x1791;
        _this.name = 'CreatorsIsEmpty';
        if (typeof Error.captureStackTrace === 'function') {
            Error.captureStackTrace(_this, CreatorsIsEmptyError);
        }
        return _this;
    }
    return CreatorsIsEmptyError;
}(Error));
exports.CreatorsIsEmptyError = CreatorsIsEmptyError;
createErrorFromCodeLookup.set(0x1791, function () { return new CreatorsIsEmptyError(); });
createErrorFromNameLookup.set('CreatorsIsEmpty', function () { return new CreatorsIsEmptyError(); });
/**
 * Attempts to resolve a custom program error from the provided error code.
 */
function errorFromCode(code) {
    var createError = createErrorFromCodeLookup.get(code);
    return createError != null ? createError() : null;
}
exports.errorFromCode = errorFromCode;
/**
 * Attempts to resolve a custom program error from the provided error name, i.e. 'Unauthorized'.
 */
function errorFromName(name) {
    var createError = createErrorFromNameLookup.get(name);
    return createError != null ? createError() : null;
}
exports.errorFromName = errorFromName;
