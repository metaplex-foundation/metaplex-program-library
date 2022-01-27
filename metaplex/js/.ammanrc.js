// @ts-check
'use strict';
const path = require('path');
const { LOCALHOST, tmpLedgerDir } = require('@metaplex-foundation/amman');

const localDeployDir = path.resolve(process.cwd(), '../..', 'target', 'deploy');

const programIds = {
  metadata: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
  vault: 'vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn',
  auction: 'auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8',
  metaplex: 'p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98',
};

function localDeployPath(programName) {
  return path.join(localDeployDir, `${programName}.so`);
}
const programs = [
  {
    programId: programIds.metadata,
    deployPath: localDeployPath('mpl_token_metadata'),
  },
  {
    programId: programIds.vault,
    deployPath: localDeployPath('mpl_token_vault'),
  },
  { programId: programIds.auction, deployPath: localDeployPath('mpl_auction') },
  {
    programId: programIds.metaplex,
    deployPath: localDeployPath('mpl_metaplex'),
  },
];

const validator = {
  killRunningValidators: true,
  programs,
  jsonRpcUrl: LOCALHOST,
  websocketUrl: '',
  commitment: 'confirmed',
  ledgerDir: tmpLedgerDir(),
  resetLedger: true,
  verifyFees: false,
};

module.exports = {
  validator,
};
