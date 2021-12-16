// @ts-check
'use strict';
const path = require('path');

const localDeployDir = path.join(__dirname, 'target', 'deploy');

const programIds = {
  metadata: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
  vault: 'vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn',
  auction: 'auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8',
  metaplex: 'p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98',
};

function localDeployPath(programName) {
  return path.join(localDeployDir, `${programName}.so`);
}
const programs = {
  metadata: { programId: programIds.metadata, deployPath: localDeployPath('mpl_token_metadata') },
  vault: { programId: programIds.vault, deployPath: localDeployPath('mpl_token_vault') },
  auction: { programId: programIds.auction, deployPath: localDeployPath('mpl_auction') },
  metaplex: { programId: programIds.mpl, deployPath: localDeployPath('mpl_metaplex') },
};

const validator = {
  verifyFees: true,
};

module.exports = {
  programs,
  validator,
};
