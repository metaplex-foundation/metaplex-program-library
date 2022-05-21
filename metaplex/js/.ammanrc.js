'use strict';
// @ts-check
const base = require('../../.base-ammanrc.js');

const validator = {
  ...base.validator,
  programs: [
    base.programs.metadata,
    base.programs.vault,
    base.programs.auction,
    base.programs.metaplex,
  ],
};
module.exports = { validator };
