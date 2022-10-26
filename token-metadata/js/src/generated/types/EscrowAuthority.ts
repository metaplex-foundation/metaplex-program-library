/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';
/**
 * This type is used to derive the {@link EscrowAuthority} type as well as the de/serializer.
 * However don't refer to it in your code but use the {@link EscrowAuthority} type instead.
 *
 * @category userTypes
 * @category enums
 * @category generated
 * @private
 */
export type EscrowAuthorityRecord = {
  TokenOwner: void /* scalar variant */;
  Creator: { fields: [web3.PublicKey] };
};

/**
 * Union type respresenting the EscrowAuthority data enum defined in Rust.
 *
 * NOTE: that it includes a `__kind` property which allows to narrow types in
 * switch/if statements.
 * Additionally `isEscrowAuthority*` type guards are exposed below to narrow to a specific variant.
 *
 * @category userTypes
 * @category enums
 * @category generated
 */
export type EscrowAuthority = beet.DataEnumKeyAsKind<EscrowAuthorityRecord>;

export const isEscrowAuthorityTokenOwner = (
  x: EscrowAuthority,
): x is EscrowAuthority & { __kind: 'TokenOwner' } => x.__kind === 'TokenOwner';
export const isEscrowAuthorityCreator = (
  x: EscrowAuthority,
): x is EscrowAuthority & { __kind: 'Creator' } => x.__kind === 'Creator';

/**
 * @category userTypes
 * @category generated
 */
export const escrowAuthorityBeet = beet.dataEnum<EscrowAuthorityRecord>([
  ['TokenOwner', beet.unit],
  [
    'Creator',
    new beet.BeetArgsStruct<EscrowAuthorityRecord['Creator']>(
      [['fields', beet.fixedSizeTuple([beetSolana.publicKey])]],
      'EscrowAuthorityRecord["Creator"]',
    ),
  ],
]) as beet.FixableBeet<EscrowAuthority>;
