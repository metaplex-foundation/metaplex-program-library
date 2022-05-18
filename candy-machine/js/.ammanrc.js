'use strict';
// @ts-check
const base = require('../../.base-ammanrc.js');
const validator = {
    ...base.validator,
    programs: [base.programs.metadata, base.programs.candy_machine],
};
module.exports = {validator};
