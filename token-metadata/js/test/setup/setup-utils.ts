import debug from 'debug';
import { tmpdir } from 'os';
import path from 'path';

export const logError = debug('mpl:setup:error');
export const logInfo = debug('mpl:setup:info');
export const logDebug = debug('mpl:setup:debug');
export const logTrace = debug('mpl:setup:trace');

export const ledgerDir = path.join(tmpdir(), 'mpl-tests-ledger');
export const projectRoot = path.resolve(__dirname, '..', '..', '..', '..');
export const localDeployDir = path.join(projectRoot, 'target', 'deploy');
export const solanaConfigPath = path.join(__dirname, '..', 'config', 'solana-validator.yml');

export const programIds = {
  metadata: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
  vault: 'vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn',
  auction: 'auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8',
  metaplex: 'p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98',
};

function localDeployPath(programName: string) {
  return path.join(localDeployDir, `${programName}.so`);
}

export const programs: Record<string, string> = {
  metadata: localDeployPath('mpl_token_metadata'),
  vault: localDeployPath('mpl_token_vault'),
  auction: localDeployPath('mpl_auction'),
  mpl: localDeployPath('mpl_metaplex'),
};
