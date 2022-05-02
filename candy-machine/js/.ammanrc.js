'use strict';
// @ts-check
const base = require('../../.ammanrc.js');
console.log(base.validator.programs.find(({label})=>label === "Candy Machine"))
const validator = {...base.validator, programs: [
  base.validator.programs.find((e)=>e.label === "Candy Machine"),
  base.validator.programs.find((e)=>e.label === "Metadata"),
]};
module.exports = {
  validator,
  relay: {
    enabled: true,
    killRunningRelay: true,
  },
};
