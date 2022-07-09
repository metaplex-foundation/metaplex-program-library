// @ts-check
const path = require('path');
const programDir = path.join(__dirname, '..', 'program');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
  idlGenerator: 'anchor',
  programName: 'auctioneer',
  programId: 'neer8g6yJq2mQM6KbnViEDAD4gr3gRZyMMf4F2p3MEh',
  idlDir,
  sdkDir,
  binaryInstallDir,
  programDir,
  typeAliases: {
    UnixTimestamp: 'i64',
  },
};
