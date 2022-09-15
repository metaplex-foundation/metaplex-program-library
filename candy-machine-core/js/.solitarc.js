// @ts-check
const path = require('path');
const programDir = path.join(__dirname, '..', 'program');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'candy_machine_core',
    programId: 'cndy3CZK71ZHMp9ddpq5NVvQDx33o6cCYDf4JBAWCk7',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
