import test from 'tape';
import spok, { Specifications } from 'spok';
import { promises as fs } from 'fs';
import path from 'path';
import { Key, Metadata, metadataBeet, TokenStandard, UseMethod } from '../src/mpl-token-metadata';
import { PublicKey } from '@solana/web3.js';
import { bignum } from '@metaplex-foundation/beet';

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
      remaining: (n: bignum) => n.toString() === '2',
      total: (n: bignum) => n.toString() === '1',
    },
  });
});
