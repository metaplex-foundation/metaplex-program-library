import { Test } from 'tape';
import { bignum, COption } from '@metaplex-foundation/beet';
import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { Specification, Specifications } from 'spok';
declare type Assert = {
    equal(actual: any, expected: any, msg?: string): void;
    deepEqual(actual: any, expected: any, msg?: string): void;
    ok(value: any, msg?: string): void;
};
export declare function assertSamePubkey(t: Assert, a: PublicKey | COption<PublicKey>, b: PublicKey): void;
export declare function spokSamePubkey(a: PublicKey | COption<PublicKey>): Specifications<PublicKey>;
export declare function spokSameBignum(a: BN | bignum): Specification<bignum>;
export declare function assertIsNotNull<T>(t: Test, x: T | null | undefined): asserts x is T;
export {};
