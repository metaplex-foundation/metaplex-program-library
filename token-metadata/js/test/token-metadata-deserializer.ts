import test from 'tape';
import spok from 'spok';
import { promises as fs } from 'fs';
import path from 'path';
import { Key, Metadata } from '../';

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
});
