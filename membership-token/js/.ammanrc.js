'use strict';
// @ts-check
const base = require('../../.ammanrc.js');

const validator = {
  ...base.validator,
  programs: [base.programs.metadata, base.programs.membershipToken],
};
module.exports = { validator };
