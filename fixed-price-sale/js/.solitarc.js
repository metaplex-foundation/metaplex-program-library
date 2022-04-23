// @ts-check
const path = require('path');
const programDir = path.join(__dirname, '..', 'program');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
  idlGenerator: 'anchor',
  programName: 'fixed_price_sale',
  programId: 'SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo',
  idlDir,
  sdkDir,
  binaryInstallDir,
  programDir,
  rustbin: {
    // NOTE: this is a workaround for missing anchor-cli version matching ~0.22
    // It should be removed as soon as 'anchor-lang' is upgraded and a matching anchor-cli
    // version exists.
    versionRangeFallback: '~0.24.2',
  },
};
