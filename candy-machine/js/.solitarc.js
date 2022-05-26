// @ts-check
const path = require('path');
const programDir = path.join(__dirname, '..', 'program');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'candy_machine',
    programId: 'cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
