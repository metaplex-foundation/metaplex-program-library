const repoWideConfig = require('../../.np-config.js');
const { name } = require('./package.json');
module.exports = { ...repoWideConfig, message: `chore: update ${name} to v%s` };
