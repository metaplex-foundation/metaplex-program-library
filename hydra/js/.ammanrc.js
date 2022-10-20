'use strict';
// @ts-check
const base = require('../../.base-ammanrc.js');

const hydraValidator = {
  programs: [base.programs.metadata, base.programs.hydra],
  commitment: 'confirmed',
  verifyFees: false,
};

const validator = {
  hydraValidator,
  programs: [base.programs.metadata, base.programs.hydra],
};
module.exports = { validator };
