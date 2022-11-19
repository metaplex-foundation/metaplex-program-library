// @ts-check
'use strict';
const path = require('path');

const localDeployDir = path.join(__dirname, 'test-programs');
const { LOCALHOST, tmpLedgerDir } = require('@metaplex-foundation/amman');

function localDeployPath(programName) {
  return path.join(localDeployDir, `${programName}.so`);
}

const programs = {
  metadata: {
    label: 'Metadata',
    programId: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
    deployPath: localDeployPath('mpl_token_metadata'),
  },
  token_sale: {
    label: 'Fixed Price Token Sale',
    programId: 'SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo',
    deployPath: localDeployPath('mpl_fixed_price_sale'),
  },
  candy_machine: {
    label: 'Candy Machine',
    programId: 'cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ',
    deployPath: localDeployPath('mpl_candy_machine'),
  },
  hydra: {
    label: 'Hydra',
    programId: 'hyDQ4Nz1eYyegS6JfenyKwKzYxRsCWCriYSAjtzP4Vg',
    deployPath: localDeployPath('mpl_hydra'),
  },
  candy_machine_core: {
    label: 'Candy Machine Core',
    programId: 'CndyV3LdqHUfDLmE5naZjVN8rBZz4tqhdefbAnjHG3JR',
    deployPath: localDeployPath('mpl_candy_machine_core'),
  },
};

const validator = {
  killRunningValidators: true,
  programs,
  commitment: 'singleGossip',
  resetLedger: true,
  verifyFees: false,
  jsonRpcUrl: LOCALHOST,
  websocketUrl: '',
  ledgerDir: tmpLedgerDir(),
};

module.exports = {
  programs,
  validator,
  relay: {
    enabled: true,
    killRunningRelay: true,
  },
};
