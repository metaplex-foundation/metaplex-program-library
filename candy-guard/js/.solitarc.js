// @ts-check
const path = require('path');
const programDir = path.join(__dirname, '..', 'program');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'candy_guard',
    programId: 'grd1hVewsa8dR1T1JfSFGzQUqgWmc1xXZ3uRRFJJ8XJ',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
