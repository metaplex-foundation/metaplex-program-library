'use strict';
// @ts-check
const base = require('../../.base-ammanrc.js');
const validator = {
    ...base.validator,
    programs: [base.programs.candy_guard],
};
module.exports = {validator};
