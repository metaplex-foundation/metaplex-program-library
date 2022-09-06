'use strict';
// @ts-check
const { LOCALHOST, tmpLedgerDir } = require('@metaplex-foundation/amman');
const base = require('../../.base-ammanrc.js');

const hydraValidator = {
  killRunningValidators: true,
  programs: [base.programs.metadata, base.programs.hydra],
  commitment: 'confirmed',
  resetLedger: true,
  verifyFees: false,
  jsonRpcUrl: LOCALHOST,
  websocketUrl: '',
  ledgerDir: tmpLedgerDir(),
};

const validator = {
  hydraValidator,
  programs: [base.programs.metadata, base.programs.hydra],
};
module.exports = { validator };
