import test from 'tape';
import spok, { Specifications } from 'spok';
import { promises as fs } from 'fs';
import path from 'path';
import {
  Key,
  keyBeet,
  Metadata,
  metadataBeet,
  TokenDelegateRole,
  tokenDelegateRoleBeet,
  TokenRecord,
  TokenStandard,
  TokenState,
  tokenStateBeet,
  UseMethod,
} from '../src/mpl-token-metadata';
import { PublicKey } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

const fixtures = path.join(__dirname, 'fixtures');

test('deserialize: faulty token metadata', async (t) => {
  const filename = 'faulty_13gxS4r6SiJn8fwizKZT2W8x8DL6vjN1nAhPWsfNXegb.buf';
  const data = await fs.readFile(path.join(fixtures, filename));
  const [metadata] = Metadata.deserialize(data);

  spok(t, metadata, {
    key: Key.MetadataV1,
    data: {
      symbol: spok.startsWith('BORYOKU'),
      name: spok.startsWith('Boryoku Dragonz #515'),
      sellerFeeBasisPoints: 500,
    },
    primarySaleHappened: true,
    isMutable: true,
    editionNonce: 255,
    tokenStandard: null,
    collection: null,
    uses: null,
  });

  {
    t.comment('+++ adding tokenStandard and corrupting following data');

    const metadataWithTokenStandard = Metadata.fromArgs({
      ...metadata,
      tokenStandard: TokenStandard.NonFungibleEdition,
    });

    const [serialized] = metadataBeet.serialize(metadataWithTokenStandard);
    const buf = Buffer.concat([serialized, Buffer.from('some bogus data here')]);
    const [deserialized] = Metadata.deserialize(buf);

    spok(t, deserialized, {
      key: Key.MetadataV1,
      data: {
        symbol: spok.startsWith('BORYOKU'),
        name: spok.startsWith('Boryoku Dragonz #515'),
        sellerFeeBasisPoints: 500,
      },
      primarySaleHappened: true,
      isMutable: true,
      editionNonce: 255,
      tokenStandard: TokenStandard.NonFungibleEdition,
      collection: null,
      uses: null,
    });
  }

  {
    t.comment('+++ adding collection and corrupting following data');

    const metadataWithTokenStandardAndCollection = Metadata.fromArgs({
      ...metadata,
      tokenStandard: TokenStandard.NonFungibleEdition,
      collection: { verified: true, key: metadata.updateAuthority },
    });

    const [serialized] = metadataBeet.serialize(metadataWithTokenStandardAndCollection);
    const buf = Buffer.concat([serialized, Buffer.from('some bogus data here')]);
    const [deserialized] = Metadata.deserialize(buf);

    spok(t, deserialized, {
      key: Key.MetadataV1,
      data: {
        symbol: spok.startsWith('BORYOKU'),
        name: spok.startsWith('Boryoku Dragonz #515'),
        sellerFeeBasisPoints: 500,
      },
      primarySaleHappened: true,
      isMutable: true,
      editionNonce: 255,
      tokenStandard: TokenStandard.NonFungibleEdition,
      collection: {
        verified: true,
        key: <Specifications<PublicKey>>((k: PublicKey) => k.equals(metadata.updateAuthority)),
      },
      uses: null,
    });
  }
});

test('deserialize: fixed token metadata', async (t) => {
  const filename = 'faulty_13gxS4r6SiJn8fwizKZT2W8x8DL6vjN1nAhPWsfNXegb.buf';
  const data = await fs.readFile(path.join(fixtures, filename));
  const [metadata] = Metadata.deserialize(data);

  const metadataFixed = Metadata.fromArgs({
    ...metadata,
    tokenStandard: TokenStandard.NonFungibleEdition,
    collection: { verified: true, key: metadata.updateAuthority },
    uses: {
      useMethod: UseMethod.Multiple,
      remaining: 2,
      total: 1,
    },
  });
  const [buf] = metadataBeet.serialize(metadataFixed);
  const [deserialized] = Metadata.deserialize(buf);

  spok(t, deserialized, {
    key: Key.MetadataV1,
    data: {
      symbol: spok.startsWith('BORYOKU'),
      name: spok.startsWith('Boryoku Dragonz #515'),
      sellerFeeBasisPoints: 500,
    },
    primarySaleHappened: true,
    isMutable: true,
    editionNonce: 255,
    tokenStandard: TokenStandard.NonFungibleEdition,
    collection: {
      verified: true,
      key: <Specifications<PublicKey>>((k: PublicKey) => k.equals(metadata.updateAuthority)),
    },
    uses: {
      useMethod: UseMethod.Multiple,
      remaining: (n: beet.bignum) => n.toString() === '2',
      total: (n: beet.bignum) => n.toString() === '1',
    },
  });
});

test('deserialize: token record without lockedTransfer', async (t) => {
  // 1 (Key)
  // 1 (bump)
  // 1 (state)
  // 9 (optional rule set revision)
  // 33 (optional delegate)
  // 2 (optional delegate role)
  const buffer = Buffer.alloc(48);
  let offset = 0;

  // key
  keyBeet.write(buffer, offset, Key.TokenRecord);
  offset += keyBeet.byteSize;

  // bump
  beet.u8.write(buffer, offset, 255);
  offset += beet.u8.byteSize;

  // state
  tokenStateBeet.write(buffer, offset, TokenState.Unlocked);
  offset += tokenStateBeet.byteSize;

  // ruleSetRevision
  const ruleSetRevisionBeet = beet.coption(beet.u64).toFixedFromValue(1);
  ruleSetRevisionBeet.write(buffer, offset, 1);
  offset += ruleSetRevisionBeet.byteSize;

  // delegate
  const delegateBeet = beet.coption(beetSolana.publicKey).toFixedFromValue(PublicKey.default);
  delegateBeet.write(buffer, offset, PublicKey.default);
  offset += delegateBeet.byteSize;

  // ruleSetRevision
  const delegateRoleBeet = beet
    .coption(tokenDelegateRoleBeet)
    .toFixedFromValue(TokenDelegateRole.Sale);
  delegateRoleBeet.write(buffer, offset, TokenDelegateRole.Sale);
  offset += delegateRoleBeet.byteSize;

  const [tokenRecord] = TokenRecord.deserialize(buffer);

  t.true(tokenRecord.lockedTransfer == null);
});

test('deserialize: failed token record without lockedTransfer', async (t) => {
  // 1 (Key)
  // 1 (bump)
  // 1 (state)
  // 9 (optional rule set revision)
  // 33 (optional delegate)
  // 2 (optional delegate role)
  // 1 extra byte (garbage)
  const buffer = Buffer.alloc(48);
  let offset = 0;

  // key
  keyBeet.write(buffer, offset, Key.TokenRecord);
  offset += keyBeet.byteSize;

  // bump
  beet.u8.write(buffer, offset, 255);
  offset += beet.u8.byteSize;

  // state
  tokenStateBeet.write(buffer, offset, TokenState.Unlocked);
  offset += tokenStateBeet.byteSize;

  // ruleSetRevision
  const ruleSetRevisionBeet = beet.coption(beet.u64).toFixedFromValue(1);
  ruleSetRevisionBeet.write(buffer, offset, 1);
  offset += ruleSetRevisionBeet.byteSize;

  // delegate
  const delegateBeet = beet.coption(beetSolana.publicKey).toFixedFromValue(PublicKey.default);
  delegateBeet.write(buffer, offset, PublicKey.default);
  offset += delegateBeet.byteSize;

  // ruleSetRevision
  const delegateRoleBeet = beet
    .coption(tokenDelegateRoleBeet)
    .toFixedFromValue(TokenDelegateRole.Sale);
  delegateRoleBeet.write(buffer, offset, TokenDelegateRole.Sale);
  offset += delegateRoleBeet.byteSize;

  // garbage byte
  beet.u8.write(buffer, offset, 255);
  offset += beet.u8.byteSize;

  let failed = false;

  try {
    TokenRecord.deserialize(buffer);
  } catch (e) {
    // we are expecting an error
    failed = true;
  }

  t.true(failed, 'deserialization failed');
});
