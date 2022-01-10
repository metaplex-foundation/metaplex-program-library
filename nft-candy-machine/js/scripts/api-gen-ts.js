// @ts-check
'use strict';

const PROGRAM_NAME = 'nft-candy-machine-v2';

const path = require('path');
const generatedIdlDir = path.join(__dirname, '..', 'idl');
const programDir = path.join(__dirname, '..', '..', 'program');
const { spawn } = require('child_process');

const anchor = spawn('anchor', ['build', '--idl', generatedIdlDir], { cwd: programDir })
  .on('error', (err) => {
    console.error(err);
    // @ts-ignore this err does have a code
    if (err.code === 'ENOENT') {
      console.error(
        'Ensure that `anchor` is installed and in your path, see:\n  https://project-serum.github.io/anchor/getting-started/installation.html#install-anchor\n',
      );
    }
    process.exit(1);
  })
  .on('exit', () => {
    console.log('IDL written to: %s', path.join(generatedIdlDir, `${PROGRAM_NAME}.json`));
    process.exit(0);
  });

anchor.stdout.on('data', (buf) => console.log(buf.toString('utf8')));
anchor.stderr.on('data', (buf) => console.error(buf.toString('utf8')));
