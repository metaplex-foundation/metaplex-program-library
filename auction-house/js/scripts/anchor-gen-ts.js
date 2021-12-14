// @ts-check
'use strict';

const path = require('path');
const generatedTsDir = path.join(__dirname, '..', 'src', 'generated');
const { execSync: exec } = require('child_process');

// TODO(thlorenz): ensure `anchor` is installed

exec(`anchor build --idl-ts ${generatedTsDir}`);
