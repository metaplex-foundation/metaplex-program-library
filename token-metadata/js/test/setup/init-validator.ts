import { execSync as exec, spawn } from 'child_process';
import { solanaConfigPath, programs, ledgerDir } from './setup-utils';
import { LOCALHOST, logError, logInfo, logTrace, programIds } from '../utils';
import { prepareConfig } from './prepare-config';
import { ensureValidatorIsUpAndChargesFees } from './validator-utils';

const PIPE_VALIDATOR = process.env.PIPE_VALIDATOR != null;

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function main() {
  logInfo('Preparing config');
  prepareConfig();

  try {
    exec('pkill solana-test-validator');
    logInfo('Killed currently running solana-test-validator');
    await sleep(1000);
  } catch (err) {}

  const args = ['-C', solanaConfigPath, '-r', '--ledger', ledgerDir];
  const programFile = programs.metadata;
  args.push('--bpf-program');
  args.push(programIds.metadata);
  args.push(programFile);

  const cmd = `solana-test-validator ${args.join(' \\\n  ')}`;
  if (logTrace.enabled) {
    logTrace('Launching validator with the following command');
    console.log(cmd);
  }
  const child = spawn('solana-test-validator', args, {
    detached: true,
    stdio: PIPE_VALIDATOR ? 'inherit' : 'ignore',
  });
  child.unref();

  logInfo(
    'Spawning new solana-test-validator with programs predeployed and ledger at %s',
    ledgerDir,
  );
  logInfo('Rerun with `PIPE_VALIDATOR=1` to triage eventual validator startup issues');

  await ensureValidatorIsUpAndChargesFees(LOCALHOST);

  logInfo('solana-test-validator is up');
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    logError(err);
    if (err.stderr != null) {
      logError(err.stderr.toString());
    }
    process.exit(1);
  });
