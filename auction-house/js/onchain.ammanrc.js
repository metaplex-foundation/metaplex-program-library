'use strict';
// @ts-check
const base = require('../../.base-ammanrc.js');

const accounts = {
  auction_house: {
    label: "Auction House",
    programId: 'hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk',
    executable: true,
  },
}

const validator = {
  ...base.validator,
  programs: [],
  accounts: accounts
};

module.exports = { validator };
