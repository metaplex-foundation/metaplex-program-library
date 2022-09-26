'use strict';
// @ts-check
const base = require('../../.base-ammanrc.js');
const validator = {
    ...base.validator,
    programs: [base.programs.metadata, base.programs.trifle],
};

const storage = {
    enabled: true,
    storageId: 'mock-storage',
    clearOnStart: true,
};

module.exports = { validator, storage };
